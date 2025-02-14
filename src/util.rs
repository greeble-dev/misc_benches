use bevy_transform::components::Transform;
use rand::{distributions::Standard, prelude::Distribution, Rng};

// Return how many values of T can comfortably fit in L1 on reasonably modern x86.
pub const fn l1_sized_count<T>() -> usize {
    (16 * 1024) / size_of::<T>()
}

// Return how many values of T can comfortably fit in L2 on AMD Zen 4.
pub const fn l2_sized_count<T>() -> usize {
    (512 * 1024) / size_of::<T>()
}

pub fn random_transform_array<R: Rng + ?Sized>(rng: &mut R, count: usize) -> Vec<Transform> {
    Standard
        .sample_iter(rng)
        .map(Transform::from_rotation)
        .take(count)
        .collect()
}

pub fn random_array<T, R: Rng + ?Sized>(rng: &mut R, count: usize) -> Vec<T>
where
    Standard: Distribution<T>,
{
    Standard.sample_iter(rng).take(count).collect()
}
