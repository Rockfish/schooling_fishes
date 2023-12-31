use glam::Vec2;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
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

pub fn rand_normal_distribution() -> f32 {
    let normal: Normal<f32> = Normal::new(2.0, 0.2).unwrap();
    let v = normal.sample(&mut thread_rng());
    v
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
        pos.x -= MaxX as f32;
    }

    if pos.x < 0.0 {
        pos.x += MaxX as f32;
    }

    if pos.y < 0.0 {
        pos.y += MaxY as f32;
    }

    if pos.y > MaxY as f32 {
        pos.y -= MaxY as f32
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{rand_normal_distribution, Truncate, WrapAround};
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

    #[test]
    pub fn test_distribution() {
        for i in 0..1000 {
            let x = rand_normal_distribution() - 2.0;
            println!("{x}");
        }
    }

    #[test]
    pub fn test_projection() {
        let wave_vec = vec2(10.0, 10.0);
        let spot_vec = vec2(9.0, 4.0);

        let proj_vec = spot_vec.project_onto(wave_vec);

        println!("{:?}", proj_vec);
    }
}
