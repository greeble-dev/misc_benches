use bevy_math::Dir3;
use bevy_transform::components::Transform;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use glam::Quat;
use misc_benches::util::*;
use rand::prelude::*;

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

trait FastRenormalize {
    fn fast_renormalize(self) -> Self;
}

impl FastRenormalize for Quat {
    fn fast_renormalize(self) -> Self {
        let length_squared = self.length_squared();
        self * (0.5 * (3.0 - length_squared))
    }
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

fn rotate_axis_normalize_fast(dst: &mut Transform, src: Transform, axis: Dir3, angle: f32) {
    *dst = src;
    dst.rotate_axis(axis, angle);
    dst.rotation = dst.rotation.fast_renormalize();
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

#[inline(never)]
fn rotate_axis_normalize_fast_outer(params: &mut RotateAxisParams) {
    rotate_axis_normalize_inner(params, rotate_axis_normalize_fast);
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

    group.bench_function(format!("count = {COUNT}, normalize = fast"), |b| {
        b.iter(|| {
            rotate_axis_normalize_fast_outer(&mut params);
        })
    });
}

fn single_normalize_false(dst: &mut Transform, src: Transform) {
    dst.rotation = src.rotation;
}

fn single_normalize_true(dst: &mut Transform, src: Transform) {
    dst.rotation = src.rotation.normalize();
}

fn single_normalize_reactive(dst: &mut Transform, src: Transform) {
    let l = src.rotation.length_squared();

    if (1.0 - l).abs() > 0.0001 {
        dst.rotation = src.rotation / l.sqrt();
    } else {
        dst.rotation = src.rotation;
    }
}

fn single_normalize_fast(dst: &mut Transform, src: Transform) {
    dst.rotation = src.rotation.fast_renormalize();
}

struct SingleNormalizeParams<'a> {
    dst_array: &'a mut [Transform],
    src_array: &'a [Transform],
}

fn single_normalize_inner<F>(params: &mut SingleNormalizeParams, f: F)
where
    F: Fn(&mut Transform, Transform),
{
    for i in 0..params.dst_array.len() {
        f(&mut params.dst_array[i], params.src_array[i]);
    }
}

#[inline(never)]
fn single_normalize_false_outer(params: &mut SingleNormalizeParams) {
    single_normalize_inner(params, single_normalize_false);
}

#[inline(never)]
fn single_normalize_true_outer(params: &mut SingleNormalizeParams) {
    single_normalize_inner(params, single_normalize_true);
}

#[inline(never)]
fn single_normalize_reactive_outer(params: &mut SingleNormalizeParams) {
    single_normalize_inner(params, single_normalize_reactive);
}

#[inline(never)]
fn single_normalize_fast_outer(params: &mut SingleNormalizeParams) {
    single_normalize_inner(params, single_normalize_fast);
}

pub fn single_normalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_normalize");

    const COUNT: usize = l1_sized_count::<(Transform, Transform)>();

    group.throughput(Throughput::Elements(COUNT as u64));

    let mut rng = StdRng::seed_from_u64(1234);

    let mut params = SingleNormalizeParams {
        dst_array: &mut vec![Transform::IDENTITY; COUNT],
        src_array: &random_transform_array(&mut rng, COUNT),
    };

    group.bench_function(format!("count = {COUNT}, normalize = false"), |b| {
        b.iter(|| {
            single_normalize_false_outer(&mut params);
        })
    });

    group.bench_function(format!("count = {COUNT}, normalize = true"), |b| {
        b.iter(|| {
            single_normalize_true_outer(&mut params);
        })
    });

    group.bench_function(format!("count = {COUNT}, normalize = reactive"), |b| {
        b.iter(|| {
            single_normalize_reactive_outer(&mut params);
        })
    });

    group.bench_function(format!("count = {COUNT}, normalize = fast"), |b| {
        b.iter(|| {
            single_normalize_fast_outer(&mut params);
        })
    });
}

criterion_group!(
    normalize,
    transform_normalize,
    rotate_axis_normalize,
    single_normalize,
);

criterion_main!(normalize);
