use rand::prelude::ThreadRng;
use rand::{thread_rng, Rng};

fn foo() {
    let mut rng = thread_rng();
    let x: u32 = rng.gen();
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
