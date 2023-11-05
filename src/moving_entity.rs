use crate::base_entity::BaseEntity;
use glam::Vec2;

#[derive(Debug)]
pub struct MovingEntity {
    pub velocity: Vec2,



    pub mass: f32,

    // the maximum speed this entity may travel at.
    pub max_speed: f32,

    // the maximum force this entity can produce to power itself
    pub max_force: f32,

    // the maximum rate (radians per second)this entity can rotate
    pub max_turn_rate: f32,
}

impl MovingEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(velocity: Vec2, max_speed: f32, mass: f32, turn_rate: f32, max_force: f32) -> Self {
        MovingEntity {
            velocity,
            mass,
            max_speed,
            max_force,
            max_turn_rate: turn_rate,
        }
    }

    pub fn MaxForce(&self) -> f32 {
        self.max_force
    }

    pub fn MaxSpeed(&self) -> f32 {
        self.max_speed
    }

    pub fn Velocity(&self) -> Vec2 {
        self.velocity
    }

    pub fn Speed(&self) -> f32 {
        self.velocity.length()
    }
}
