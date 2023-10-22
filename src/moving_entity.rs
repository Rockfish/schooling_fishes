use crate::base_entity::BaseGameEntity;
use glam::Vec2;

#[derive(Debug)]
pub struct MovingEntity {
    pub m_vVelocity: Vec2,

    //a normalized vector pointing in the direction the entity is heading.
    pub m_vHeading: Vec2,

    //a vector perpendicular to the heading vector
    pub m_vSide: Vec2,

    pub m_dMass: f32,

    //the maximum speed this entity may travel at.
    pub m_dMaxSpeed: f32,

    //the maximum force this entity can produce to power itself
    //(think rockets and thrust)
    pub m_dMaxForce: f32,

    //the maximum rate (radians per second)this vehicle can rotate
    pub m_dMaxTurnRate: f32,
}

impl MovingEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(velocity: Vec2, max_speed: f32, heading: Vec2, mass: f32, turn_rate: f32, max_force: f32) -> Self {
        MovingEntity {
            m_vVelocity: velocity,
            m_vHeading: heading,
            m_vSide: heading.perp(),
            m_dMass: mass,
            m_dMaxSpeed: max_speed,
            m_dMaxForce: max_force,
            m_dMaxTurnRate: turn_rate,
        }
    }

    pub fn MaxForce(&self) -> f32 {
        self.m_dMaxForce
    }

    pub fn MaxSpeed(&self) -> f32 {
        self.m_dMaxSpeed
    }

    pub fn Velocity(&self) -> Vec2 {
        self.m_vVelocity
    }

    pub fn Speed(&self) -> f32 {
        self.m_vVelocity.length()
    }

    pub fn Heading(&self) -> Vec2 {
        self.m_vHeading
    }
}
