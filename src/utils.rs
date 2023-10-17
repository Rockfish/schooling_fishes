use glam::Vec2;
use rand::{Rng, thread_rng};
use std::cmp::Ordering;
use std::ops::Mul;

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

pub fn RandInt(x: i32, y: i32) -> i32 {
    thread_rng().gen_range(x..=y)
}

pub fn RandFloat() -> f32 {
    thread_rng().gen_range(0.0..=1.0)
}

pub fn RandInRange(x: f32, y: f32) -> f32 {
    thread_rng().gen_range(x..=y)
}

pub fn RandBool() -> bool {
    thread_rng().gen_range(0.0..=1.0) > 0.5
}

//returns a random float in the range -1 < n < 1
pub fn RandomClamped() -> f32 {
    thread_rng().gen_range(-1.0..=1.0)
}

pub fn Truncate(v: Vec2, max: f32) -> Vec2 {
    if v.length() > max {
        let v = v.normalize_or_zero();
        return v.mul(max);
    }
    v
}

pub fn WrapAround(pos: &mut Vec2, MaxX: i32, MaxY: i32) {
    if pos.x > MaxX as f32 {
        pos.x = 0.0;
    }

    if pos.x < 0.0 {
        pos.x = MaxX as f32;
    }

    if pos.y < 0.0 {
        pos.y = MaxY as f32;
    }

    if pos.y > MaxY as f32 {
        pos.y = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{Truncate, WrapAround};
    use glam::vec2;

    #[test]
    pub fn test_truncate() {
        let mut v = vec2(100.0, 100.0);
        println!("length: {}", v.length());

        v = Truncate(v, 5.0);
        println!("vec: {:?}  length: {}", v, v.length());
    }

    #[test]
    pub fn test_wraparound() {
        let mut v = vec2(10.0, 10.0);
        WrapAround(&mut v, 8, 8);
        println!("{:?}", v);
        let mut v = vec2(10.0, 10.0);
        WrapAround(&mut v, 10, 11);
        println!("{:?}", v);
    }
}
