use crate::game_world::GameWorld;
use crate::smoother::Smoother;
use crate::steering_behavior::SteeringBehavior;
use glam::Vec2;

pub struct Vehicle {
    //a pointer to the world data. So a vehicle can access any obstacle,
    //path, wall or agent data
    m_pWorld: &'static GameWorld,

    //the steering behavior class
    m_pSteering: SteeringBehavior,

    //some steering behaviors give jerky looking movement. The
    //following members are used to smooth the vehicle's heading
    m_pHeadingSmoother: Smoother<Vec2>,

    //this vector represents the average of the vehicle's heading
    //vector smoothed over the last few frames
    m_vSmoothedHeading: Vec2,

    //when true, smoothing is active
    m_bSmoothingOn: bool,

    //keeps a track of the most recent update time. (some of the
    //steering behaviors make use of this - see Wander)
    m_dTimeElapsed: f32,

    //buffer for the vehicle shape
    m_vecVehicleVB: Vec<Vec2>,
}

impl Vehicle {
    pub fn new(
        world: &GameWorld,
        position: Vec2,
        rotation: f32,
        velocity: Vec2,
        mass: f32,
        max_force: f32,
        max_speed: f32,
        max_turn_rate: f32,
        scale: f32,
    ) {
    }
}
