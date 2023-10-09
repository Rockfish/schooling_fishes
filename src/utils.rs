use std::cmp::Ordering;
use rand::prelude::ThreadRng;
use rand::Rng;


#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

pub fn f32_max(a: f32, b: f32) -> f32 {
    match a.partial_cmp(&b).unwrap() {
        Ordering::Less => b,
        Ordering::Equal => b,
        Ordering::Greater => a,
    }
}
pub fn RandInt(rng: &mut ThreadRng, x: i32, y: i32) -> i32 {
    rng.gen_range(x..=y)
}

pub fn RandFloat(rng: &mut ThreadRng) -> f32 {
    rng.gen_range(0.0..=1.0)
}

pub fn RandInRange(mut rng: ThreadRng, x: f32, y: f32) -> f32 {
    rng.gen_range(x..=y)
}

pub fn RandBool(rng: &mut ThreadRng) -> bool {
    if rng.gen_range(0.0..=1.0) > 0.5 {
        true
    } else {
        false
    }
}

//returns a random float in the range -1 < n < 1
pub fn RandomClamped(rng: &mut ThreadRng) -> f32 {
    rng.gen_range(-1.0..=1.0)
}
