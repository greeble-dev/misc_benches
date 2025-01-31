use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use std::{iter::repeat_with, num::NonZero, thread, time::Duration};
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

criterion_group!(benches, system, memcpy, rand,);

criterion_main!(benches);
