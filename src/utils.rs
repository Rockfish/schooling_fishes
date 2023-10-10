use std::cmp::Ordering;
use std::ops::Mul;
use glam::Vec2;
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

pub fn Truncate(v: Vec2, max: f32) -> Vec2 {
    if v.length() > max {
        let mut v = v.normalize();
        v.mul(max);
        return v;
    }
    v
}

pub fn WrapAround(pos: &mut Vec2, MaxX: i32, MaxY: i32)
{
    if pos.x > MaxX as f32
    {
        pos.x = 0.0;
    }

    if pos.x < 0.0 { pos.x = MaxX as f32; }

    if pos.y < 0.0 { pos.y = MaxY as f32; }

    if pos.y > MaxY as f32 { pos.y = 0.0; }
}

#[cfg(test)]
mod tests {
    use glam::vec2;
    use crate::utils::{Truncate, WrapAround};

    #[test]
    pub fn test_truncate() {
        let mut v = vec2(100.0, 100.0);
        println!("length: {}", v.length());

        v = Truncate(v, 5.0);
        println!("length: {}", v.length());
    }

    #[test]
    pub fn test_wraparound() {
        let mut v = vec2(10.0, 10.0);
        WrapAround(&mut v, 8, 8);
        println!("{:?}", v);
    }
}

