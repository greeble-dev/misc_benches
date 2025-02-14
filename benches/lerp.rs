use std::{f32::consts::TAU, iter::repeat_with};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use glam::{Quat, Vec4};
use misc_benches::util::*;
use rand::prelude::*;

fn random_quat<R: Rng + ?Sized>(rng: &mut R) -> Quat {
    let r0 = rng.gen_range(0.0f32..TAU);
    let r1 = rng.gen_range(0.0f32..TAU);
    let r2 = rng.gen_range(0.0f32..1.0f32);

    let (s0, c0) = r0.sin_cos();
    let (s1, c1) = r1.sin_cos();

    let t0 = (1.0 - r2).sqrt();
    let t1 = r2.sqrt();

    Quat::from_xyzw(t0 * s0, t0 * c0, t1 * s1, t1 * c1)
}

fn random_quat_array<R: Rng + ?Sized>(rng: &mut R, count: usize) -> Vec<Quat> {
    repeat_with(|| random_quat(rng)).take(count).collect()
}

/// Return two arrays of quats, where each quat has a 50/50 chance of being
/// a duplicate of the other array's quat. This is intended to stress the
/// `if dot > DOT_THRESHOLD` branch in slerp.
fn random_duplicate_quat_arrays<R: Rng + ?Sized>(rng: &mut R, count: usize) -> [Vec<Quat>; 2] {
    let mut l = random_quat_array(rng, count);
    let r = random_quat_array(rng, count);

    l.iter_mut().zip(r.iter()).for_each(|(l, &r)| {
        if rng.gen() {
            *l = r;
        }
    });

    [l, r]
}

// Return an array of quats where each component is positive. This is intended
// to avoid stressing the `if dot < 0.0` branch in slerp.
fn random_positive_quat_array<R: Rng + ?Sized>(rng: &mut R, count: usize) -> Vec<Quat> {
    random_quat_array(rng, count)
        .iter()
        .map(|q| Quat::from_xyzw(q.x.abs(), q.y.abs(), q.z.abs(), q.w.abs()))
        .collect()
}

fn quat_lerp(l: Quat, r: Quat, a: f32) -> Quat {
    Quat::from_vec4(Vec4::from(l).lerp(Vec4::from(r), a))
}

fn quat_nlerp(l: Quat, r: Quat, a: f32) -> Quat {
    l.lerp(r, a)
}

fn quat_slerp(l: Quat, r: Quat, a: f32) -> Quat {
    l.slerp(r, a)
}

struct QuatParams<'a> {
    dst: &'a mut [Quat],
    src_quat: &'a [&'a [Quat]; 2],
    src_alpha: f32,
}

fn quat_func<F>(params: &mut QuatParams, f: F)
where
    F: Fn(Quat, Quat, f32) -> Quat,
{
    for ((dst, l), r) in params
        .dst
        .iter_mut()
        .zip(params.src_quat[0].iter())
        .zip(params.src_quat[1].iter())
    {
        *dst = f(*l, *r, params.src_alpha);
    }
}

#[inline(never)]
fn quat_loop_lerp(params: &mut QuatParams) {
    quat_func(params, quat_lerp);
}

#[inline(never)]
fn quat_loop_nlerp(params: &mut QuatParams) {
    quat_func(params, quat_nlerp);
}

#[inline(never)]
fn quat_loop_slerp(params: &mut QuatParams) {
    quat_func(params, quat_slerp);
}

pub fn quat(c: &mut Criterion) {
    let mut group = c.benchmark_group("quat");

    let l1 = l1_sized_count::<(Quat, Quat, Quat)>();
    let l2 = l2_sized_count::<(Quat, Quat, Quat)>();

    for count in [l1, l2] {
        group.throughput(Throughput::Elements(count as u64));

        let mut rng = StdRng::seed_from_u64(1234);

        let mut params = QuatParams {
            dst: &mut vec![Quat::IDENTITY; count],
            src_quat: &[
                &random_quat_array(&mut rng, count),
                &random_quat_array(&mut rng, count),
            ],
            src_alpha: 0.5,
        };

        let src_quat_duplicates = random_duplicate_quat_arrays(&mut rng, count);

        let mut params_duplicates = QuatParams {
            dst: &mut vec![Quat::IDENTITY; count],
            src_quat: &[
                // TODO: Is there a better way to make these Vecs into slices?
                src_quat_duplicates[0].as_slice(),
                src_quat_duplicates[1].as_slice(),
            ],
            src_alpha: 0.5,
        };

        let mut params_positive = QuatParams {
            dst: &mut vec![Quat::IDENTITY; count],
            src_quat: &[
                &random_positive_quat_array(&mut rng, count),
                &random_positive_quat_array(&mut rng, count),
            ],
            src_alpha: 0.5,
        };

        group.bench_function(format!("count = {count}, lerp"), |b| {
            b.iter(|| {
                quat_loop_lerp(&mut params);
            })
        });

        group.bench_function(format!("count = {count}, nlerp"), |b| {
            b.iter(|| {
                quat_loop_nlerp(&mut params);
            })
        });

        group.bench_function(format!("count = {count}, slerp"), |b| {
            b.iter(|| {
                quat_loop_slerp(&mut params);
            })
        });

        group.bench_function(format!("count = {count}, slerp (duplicates)"), |b| {
            b.iter(|| {
                quat_loop_slerp(&mut params_duplicates);
            })
        });

        group.bench_function(format!("count = {count}, slerp (positive)"), |b| {
            b.iter(|| {
                quat_loop_slerp(&mut params_positive);
            })
        });
    }
}

criterion_group!(lerp, quat);

criterion_main!(lerp);
