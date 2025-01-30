use std::{iter::repeat_with, mem::size_of, num::NonZero, thread, time::Duration};

use bevy_math::Dir3;
use bevy_transform::components::Transform;
use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use rand::{distributions::Standard, prelude::*};

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

fn transform_normalize_inner<F>(dst: &mut [Transform], src: &[&[Transform]; 2], f: F)
where
    F: Fn(&Transform, &Transform) -> Transform,
{
    for i in 0..dst.len() {
        dst[i] = f(&src[0][i], &src[1][i]);
    }
}

#[inline(never)]
fn transform_normalize_false(dst: &mut [Transform], src: &[&[Transform]; 2]) {
    transform_normalize_inner(dst, src, mul_normalize_false);
}

#[inline(never)]
fn transform_normalize_true(dst: &mut [Transform], src: &[&[Transform]; 2]) {
    transform_normalize_inner(dst, src, mul_normalize_true);
}

pub fn transform_normalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_normalize");

    const COUNT: usize = l1_sized_count::<(Transform, Transform, Transform)>();

    group.throughput(Throughput::Elements(COUNT as u64));

    let mut rng = StdRng::seed_from_u64(1234);

    let mut dst = vec![Transform::IDENTITY; COUNT];
    let src = [
        random_transform_array(&mut rng, COUNT),
        random_transform_array(&mut rng, COUNT),
    ];

    group.bench_function(format!("count = {COUNT}, normalize = false"), |b| {
        b.iter(|| {
            transform_normalize_false(&mut dst, &[&src[0], &src[1]]);
        })
    });

    group.bench_function(format!("count = {COUNT}, normalize = true"), |b| {
        b.iter(|| {
            transform_normalize_true(&mut dst, &[&src[0], &src[1]]);
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

fn rotate_axis_normalize_inner<F>(
    dst_array: &mut [Transform],
    src_array: &[Transform],
    axis_array: &[Dir3],
    angle_array: &[f32],
    f: F,
) where
    F: Fn(&mut Transform, Transform, Dir3, f32),
{
    for i in 0..dst_array.len() {
        f(
            &mut dst_array[i],
            src_array[i],
            axis_array[i],
            angle_array[i],
        );
    }
}

#[inline(never)]
fn rotate_axis_normalize_false_outer(
    dst_array: &mut [Transform],
    src_array: &[Transform],
    axis_array: &[Dir3],
    angle_array: &[f32],
) {
    rotate_axis_normalize_inner(
        dst_array,
        src_array,
        axis_array,
        angle_array,
        rotate_axis_normalize_false,
    );
}

#[inline(never)]
fn rotate_axis_normalize_true_outer(
    dst_array: &mut [Transform],
    src_array: &[Transform],
    axis_array: &[Dir3],
    angle_array: &[f32],
) {
    rotate_axis_normalize_inner(
        dst_array,
        src_array,
        axis_array,
        angle_array,
        rotate_axis_normalize_true,
    );
}

#[inline(never)]
fn rotate_axis_normalize_reactive_outer(
    dst_array: &mut [Transform],
    src_array: &[Transform],
    axis_array: &[Dir3],
    angle_array: &[f32],
) {
    rotate_axis_normalize_inner(
        dst_array,
        src_array,
        axis_array,
        angle_array,
        rotate_axis_normalize_reactive,
    );
}

pub fn rotate_axis_normalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("rotate_axis_normalize");

    const COUNT: usize = l1_sized_count::<(Transform, Transform, Dir3, f32)>();

    group.throughput(Throughput::Elements(COUNT as u64));

    let mut rng = StdRng::seed_from_u64(1234);

    let mut dst_array = vec![Transform::IDENTITY; COUNT];
    let src_array = random_transform_array(&mut rng, COUNT);
    let axis_array: Vec<Dir3> = random_array(&mut rng, COUNT);
    let angle_array: Vec<f32> = random_array(&mut rng, COUNT);

    group.bench_function(format!("count = {COUNT}, normalize = false"), |b| {
        b.iter(|| {
            rotate_axis_normalize_false_outer(
                &mut dst_array,
                &src_array,
                &axis_array,
                &angle_array,
            );
        })
    });

    group.bench_function(format!("count = {COUNT}, normalize = true"), |b| {
        b.iter(|| {
            rotate_axis_normalize_true_outer(&mut dst_array, &src_array, &axis_array, &angle_array);
        })
    });

    group.bench_function(format!("count = {COUNT}, normalize = reactive"), |b| {
        b.iter(|| {
            rotate_axis_normalize_reactive_outer(
                &mut dst_array,
                &src_array,
                &axis_array,
                &angle_array,
            );
        })
    });
}

criterion_group!(
    benches,
    memcpy,
    rand,
    transform_normalize,
    rotate_axis_normalize
);

criterion_main!(benches);
