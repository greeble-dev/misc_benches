use bevy_math::prelude::*;
use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use misc_benches::util::*;
use rand::{rngs::StdRng, SeedableRng};

////////////////////////////////////////////////////////////////////////////////

#[inline(never)]
fn internal_smoothstep_noinline(t: f32) -> f32 {
    (3.0 - (2.0 * t)) * t * t
}

////////////////////////////////////////////////////////////////////////////////

struct SmoothstepParams<'a> {
    dst_array: &'a mut [f32],
    src_array: &'a [f32],
}

#[inline(never)]
fn smoothstep_explicit(params: &mut SmoothstepParams) {
    for i in 0..params.dst_array.len() {
        let t = params.src_array[i];

        params.dst_array[i] = (3.0 - (2.0 * t)) * t * t;
    }
}

#[inline(never)]
fn smoothstep_unit(params: &mut SmoothstepParams) {
    let f = SmoothStep;

    for i in 0..params.dst_array.len() {
        let t = params.src_array[i];

        params.dst_array[i] = f.sample_unchecked(t);
    }
}

#[inline(never)]
fn smoothstep_noinline(params: &mut SmoothstepParams) {
    for i in 0..params.dst_array.len() {
        let t = params.src_array[i];

        params.dst_array[i] = internal_smoothstep_noinline(t);
    }
}

#[inline(never)]
fn smoothstep_enum(params: &mut SmoothstepParams) {
    let f = EaseFunction::SmoothStep;

    for i in 0..params.dst_array.len() {
        let t = params.src_array[i];

        params.dst_array[i] = f.sample_unchecked(t);
    }
}

pub fn smoothstep(c: &mut Criterion) {
    let mut group = c.benchmark_group("smoothstep");

    const COUNT: usize = 32 * 1024;

    group.throughput(Throughput::Elements(COUNT as u64));
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(1000));

    let mut rng = StdRng::seed_from_u64(1234);

    let mut params = SmoothstepParams {
        dst_array: &mut vec![0.0f32; COUNT],
        src_array: &random_array(&mut rng, COUNT),
    };

    group.bench_function("explicit", |b| {
        b.iter(|| {
            smoothstep_explicit(&mut params);
        })
    });

    group.bench_function("unit", |b| {
        b.iter(|| {
            smoothstep_unit(&mut params);
        })
    });

    group.bench_function("noinline", |b| {
        b.iter(|| {
            smoothstep_noinline(&mut params);
        })
    });

    group.bench_function("enum", |b| {
        b.iter(|| {
            smoothstep_enum(&mut params);
        })
    });
}

////////////////////////////////////////////////////////////////////////////////

struct SmoothstepIndirectParams<'a> {
    dst_array: &'a mut [f32],
    src_array: &'a [f32],
    index_array: &'a [usize],
}

#[inline(never)]
fn smoothstep_indirect_explicit(params: &mut SmoothstepIndirectParams) {
    for i in 0..params.dst_array.len() {
        let t = params.src_array[params.index_array[i]];

        params.dst_array[i] = (3.0 - (2.0 * t)) * t * t;
    }
}

#[inline(never)]
fn smoothstep_indirect_unit(params: &mut SmoothstepIndirectParams) {
    let f = SmoothStep;

    for i in 0..params.dst_array.len() {
        let t = params.src_array[params.index_array[i]];

        params.dst_array[i] = f.sample_unchecked(t);
    }
}

#[inline(never)]
fn smoothstep_indirect_noinline(params: &mut SmoothstepIndirectParams) {
    for i in 0..params.dst_array.len() {
        let t = params.src_array[params.index_array[i]];

        params.dst_array[i] = internal_smoothstep_noinline(t);
    }
}

#[inline(never)]
fn smoothstep_indirect_enum(params: &mut SmoothstepIndirectParams) {
    let f = EaseFunction::SmoothStep;

    for i in 0..params.dst_array.len() {
        let t = params.src_array[params.index_array[i]];

        params.dst_array[i] = f.sample_unchecked(t);
    }
}

pub fn smoothstep_indirect(c: &mut Criterion) {
    let mut group = c.benchmark_group("smoothstep");

    const COUNT: usize = 4 * 1024;

    group.throughput(Throughput::Elements(COUNT as u64));
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(1000));

    let mut rng = StdRng::seed_from_u64(1234);

    let index_array = random_array::<usize>(&mut rng, COUNT)
        .iter()
        .map(|i| i.rem_euclid(COUNT))
        .collect::<Vec<_>>();

    let mut params = SmoothstepIndirectParams {
        dst_array: &mut vec![0.0f32; COUNT],
        src_array: &random_array(&mut rng, COUNT),
        index_array: &index_array,
    };

    group.bench_function("explicit", |b| {
        b.iter(|| {
            smoothstep_indirect_explicit(&mut params);
        })
    });

    group.bench_function("unit", |b| {
        b.iter(|| {
            smoothstep_indirect_unit(&mut params);
        })
    });

    group.bench_function("noinline", |b| {
        b.iter(|| {
            smoothstep_indirect_noinline(&mut params);
        })
    });

    group.bench_function("enum", |b| {
        b.iter(|| {
            smoothstep_indirect_enum(&mut params);
        })
    });
}

////////////////////////////////////////////////////////////////////////////////

criterion_group!(easing, smoothstep, smoothstep_indirect);

criterion_main!(easing);
