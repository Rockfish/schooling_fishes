use crate::base_entity::BaseGameEntity;
use glam::Vec2;

pub struct MovingEntity {
    m_vVelocity: Vec2,

    //a normalized vector pointing in the direction the entity is heading.
    m_vHeading: Vec2,

    //a vector perpendicular to the heading vector
    m_vSide: Vec2,

    m_dMass: f32,

    //the maximum speed this entity may travel at.
    m_dMaxSpeed: f32,

    //the maximum force this entity can produce to power itself
    //(think rockets and thrust)
    m_dMaxForce: f32,

    //the maximum rate (radians per second)this vehicle can rotate
    m_dMaxTurnRate: f32,

    pub base_entity: BaseGameEntity,
}

impl MovingEntity {
    pub fn new(
        position: Vec2,
        radius: f32,
        velocity: Vec2,
        max_speed: f32,
        heading: Vec2,
        mass: f32,
        scale: Vec2,
        turn_rate: f32,
        max_force: f32,
    ) -> Self {
        let mut base_entity = BaseGameEntity::with_type_and_position(0, position, radius);
        base_entity.m_vScale = scale;

        MovingEntity {
            m_vVelocity: velocity,
            m_vHeading: heading,
            m_vSide: heading.perp(),
            m_dMass: mass,
            m_dMaxSpeed: max_speed,
            m_dMaxForce: max_force,
            m_dMaxTurnRate: turn_rate,
            base_entity,
        }
    }
}
