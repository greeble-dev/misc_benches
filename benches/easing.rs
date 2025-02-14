use criterion::{criterion_group, criterion_main, Criterion};

// Disabled as EaseFunction::SmoothStep requires Bevy 0.16.
/*
use bevy_math::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use misc_benches::util::*;
use rand::{rngs::StdRng, SeedableRng};

struct SmoothstepParams<'a> {
    dst_array: &'a mut [f32],
    src_array: &'a [f32],
}


#[inline(never)]
fn smoothstep_low(params: &mut SmoothstepParams) {
    for i in 0..params.dst_array.len() {
        let t = params.src_array[i];

        params.dst_array[i] = (3.0 - (2.0 * t)) * t * t;
    }
}

#[inline(never)]
fn smoothstep_high_hoisted(params: &mut SmoothstepParams) {
    let f = EasingCurve::new(0.0, 1.0, EaseFunction::SmoothStep);

    for i in 0..params.dst_array.len() {
        let t = params.src_array[i];

        params.dst_array[i] = f.sample_unchecked(t);
    }
}

#[inline(never)]
fn smoothstep_high_inside(params: &mut SmoothstepParams) {
    for i in 0..params.dst_array.len() {
        let t = params.src_array[i];

        params.dst_array[i] =
            EasingCurve::new(0.0, 1.0, EaseFunction::SmoothStep).sample_unchecked(t);
    }
}
*/

pub fn smoothstep(_: &mut Criterion) {
    /*
        let mut group = c.benchmark_group("smoothstep");

        const COUNT: usize = 4 * 1024;

        group.throughput(Throughput::Elements(COUNT as u64));

        let mut rng = StdRng::seed_from_u64(1234);

        let mut params = SmoothstepParams {
            dst_array: &mut vec![0.0f32; COUNT],
            src_array: &random_array(&mut rng, COUNT),
        };

        group.bench_function("low", |b| {
            b.iter(|| {
                smoothstep_low(&mut params);
            })
        });

        group.bench_function("high_hoisted", |b| {
            b.iter(|| {
                smoothstep_high_hoisted(&mut params);
            })
        });

        group.bench_function("high_inside", |b| {
            b.iter(|| {
                smoothstep_high_inside(&mut params);
            })
        });
    */
}

criterion_group!(easing, smoothstep);

criterion_main!(easing);
