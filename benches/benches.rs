use bevy_math::Dir3;
use bevy_transform::components::Transform;
use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use rand::{distributions::Standard, prelude::*};
use std::{iter::repeat_with, mem::size_of, num::NonZero, thread, time::Duration};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

pub fn system(_: &mut Criterion) {
    let sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::new())
            .with_memory(MemoryRefreshKind::new().with_ram()),
    );

    println!(
        "os: {} / {} / {}",
        System::long_os_version().unwrap_or("not available".to_string()),
        System::kernel_version().unwrap_or("not available".to_string()),
        System::cpu_arch().unwrap_or("not available".to_string()),
    );

    println!(
        "cpu: {}",
        sys.cpus()
            .first()
            .map(|cpu| cpu.brand().trim().to_string())
            .unwrap_or("not available".to_string())
    );

    println!(
        "cores: {}",
        sys.physical_core_count()
            .map(|cores| cores.to_string())
            .unwrap_or("not available".to_string()),
    );

    println!(
        "mem: {:.1} GB",
        sys.total_memory() as f64 * (1.0 / (1024.0 * 1024.0 * 1024.0))
    );
}

#[inline(never)]
fn memcpy_inner(dst: &mut [u8], src: &[u8]) {
    dst.clone_from_slice(src);
}

pub fn memcpy(c: &mut Criterion) {
    let mut group = c.benchmark_group("memcpy");

    let sizes = [
        ("L1", 16 * 1024),
        ("L2", 512 * 1024),
        ("L3", 16 * 1024 * 1024),
        ("RAM", 512 * 1024 * 1024),
    ];

    for (name, size) in sizes {
        group.throughput(Throughput::Bytes(size as u64));

        let mut v1 = vec![0u8; size / 2];
        let v2 = vec![0u8; size / 2];

        group.bench_function(format!("memcpy = {name}"), |b| {
            b.iter(|| memcpy_inner(&mut v1, &v2))
        });
    }
}

#[inline(never)]
fn rand_inner(iterations: u64) {
    let mut rng = StdRng::seed_from_u64(1234);

    for _ in 0..iterations {
        rng.next_u64();
    }

    criterion::black_box(rng.next_u64());
}

pub fn rand(c: &mut Criterion) {
    let mut group = c.benchmark_group("rand");

    const ITERATIONS: u64 = 100_000_000;

    group.throughput(Throughput::Elements(ITERATIONS));
    group.measurement_time(Duration::from_secs(4));
    group.warm_up_time(Duration::from_secs(2));
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);

    let max_thread_count = thread::available_parallelism()
        .map(NonZero::<usize>::get)
        .unwrap_or(1);

    for thread_count in 1..=max_thread_count {
        group.bench_function(format!("threads = {}", thread_count), |b| {
            b.iter(|| {
                let threads = repeat_with(|| thread::spawn(|| rand_inner(ITERATIONS)))
                    .take(thread_count)
                    .collect::<Vec<_>>();

                threads.into_iter().for_each(|t| t.join().unwrap());
            })
        });
    }
}

// Return how many values of T can comfortably fit in L1.
const fn l1_sized_count<T>() -> usize {
    (16 * 1024) / size_of::<T>()
}

fn random_transform_array<R: Rng + ?Sized>(rng: &mut R, count: usize) -> Vec<Transform> {
    Standard
        .sample_iter(rng)
        .map(Transform::from_rotation)
        .take(count)
        .collect()
}

fn random_array<T, R: Rng + ?Sized>(rng: &mut R, count: usize) -> Vec<T>
where
    Standard: Distribution<T>,
{
    Standard.sample_iter(rng).take(count).collect()
}

fn mul_normalize_false(l: &Transform, r: &Transform) -> Transform {
    Transform {
        translation: l.transform_point(r.translation),
        rotation: l.rotation * r.rotation,
        scale: l.scale * r.scale,
    }
}

fn mul_normalize_true(l: &Transform, r: &Transform) -> Transform {
    Transform {
        translation: l.transform_point(r.translation),
        rotation: (l.rotation * r.rotation).normalize(),
        scale: l.scale * r.scale,
    }
}

struct TransformNormalizeParams<'a> {
    dst: &'a mut [Transform],
    src: &'a [&'a [Transform]; 2],
}

fn transform_normalize_inner<F>(params: &mut TransformNormalizeParams, f: F)
where
    F: Fn(&Transform, &Transform) -> Transform,
{
    for i in 0..params.dst.len() {
        params.dst[i] = f(&params.src[0][i], &params.src[1][i]);
    }
}

#[inline(never)]
fn transform_normalize_false(params: &mut TransformNormalizeParams) {
    transform_normalize_inner(params, mul_normalize_false);
}

#[inline(never)]
fn transform_normalize_true(params: &mut TransformNormalizeParams) {
    transform_normalize_inner(params, mul_normalize_true);
}

pub fn transform_normalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_normalize");

    const COUNT: usize = l1_sized_count::<(Transform, Transform, Transform)>();

    group.throughput(Throughput::Elements(COUNT as u64));

    let mut rng = StdRng::seed_from_u64(1234);

    let mut params = TransformNormalizeParams {
        dst: &mut vec![Transform::IDENTITY; COUNT],
        src: &[
            &random_transform_array(&mut rng, COUNT),
            &random_transform_array(&mut rng, COUNT),
        ],
    };

    group.bench_function(format!("count = {COUNT}, normalize = false"), |b| {
        b.iter(|| {
            transform_normalize_false(&mut params);
        })
    });

    group.bench_function(format!("count = {COUNT}, normalize = true"), |b| {
        b.iter(|| {
            transform_normalize_true(&mut params);
        })
    });
}

fn rotate_axis_normalize_false(dst: &mut Transform, src: Transform, axis: Dir3, angle: f32) {
    *dst = src;
    dst.rotate_axis(axis, angle);
}

fn rotate_axis_normalize_true(dst: &mut Transform, src: Transform, axis: Dir3, angle: f32) {
    *dst = src;
    dst.rotate_axis(axis, angle);
    dst.rotation = dst.rotation.normalize();
}

fn rotate_axis_normalize_reactive(dst: &mut Transform, src: Transform, axis: Dir3, angle: f32) {
    *dst = src;
    dst.rotate_axis(axis, angle);

    let l = dst.rotation.length_squared();

    if (1.0 - l).abs() > 0.0001 {
        dst.rotation = dst.rotation / l.sqrt();
    }
}

struct RotateAxisParams<'a> {
    dst_array: &'a mut [Transform],
    src_array: &'a [Transform],
    axis_array: &'a [Dir3],
    angle_array: &'a [f32],
}

fn rotate_axis_normalize_inner<F>(params: &mut RotateAxisParams, f: F)
where
    F: Fn(&mut Transform, Transform, Dir3, f32),
{
    for i in 0..params.dst_array.len() {
        f(
            &mut params.dst_array[i],
            params.src_array[i],
            params.axis_array[i],
            params.angle_array[i],
        );
    }
}

#[inline(never)]
fn rotate_axis_normalize_false_outer(params: &mut RotateAxisParams) {
    rotate_axis_normalize_inner(params, rotate_axis_normalize_false);
}

#[inline(never)]
fn rotate_axis_normalize_true_outer(params: &mut RotateAxisParams) {
    rotate_axis_normalize_inner(params, rotate_axis_normalize_true);
}

#[inline(never)]
fn rotate_axis_normalize_reactive_outer(params: &mut RotateAxisParams) {
    rotate_axis_normalize_inner(params, rotate_axis_normalize_reactive);
}

pub fn rotate_axis_normalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("rotate_axis_normalize");

    const COUNT: usize = l1_sized_count::<(Transform, Transform, Dir3, f32)>();

    group.throughput(Throughput::Elements(COUNT as u64));

    let mut rng = StdRng::seed_from_u64(1234);

    let mut params = RotateAxisParams {
        dst_array: &mut vec![Transform::IDENTITY; COUNT],
        src_array: &random_transform_array(&mut rng, COUNT),
        axis_array: &random_array(&mut rng, COUNT),
        angle_array: &random_array(&mut rng, COUNT),
    };

    group.bench_function(format!("count = {COUNT}, normalize = false"), |b| {
        b.iter(|| {
            rotate_axis_normalize_false_outer(&mut params);
        })
    });

    group.bench_function(format!("count = {COUNT}, normalize = true"), |b| {
        b.iter(|| {
            rotate_axis_normalize_true_outer(&mut params);
        })
    });

    group.bench_function(format!("count = {COUNT}, normalize = reactive"), |b| {
        b.iter(|| {
            rotate_axis_normalize_reactive_outer(&mut params);
        })
    });
}

criterion_group!(
    benches,
    system,
    memcpy,
    rand,
    transform_normalize,
    rotate_axis_normalize
);

criterion_main!(benches);
