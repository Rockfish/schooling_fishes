use std::cell::RefCell;
use std::rc::Rc;
use crate::game_world::GameWorld;
use crate::smoother::Smoother;
use crate::steering_behavior::SteeringBehavior;
use glam::{Vec2, vec2};
use crate::base_entity::EntityBase;
use crate::moving_entity::MovingEntity;
use crate::param_loader::PRM;

pub struct Vehicle {
    //a pointer to the world data. So a vehicle can access any obstacle,
    //path, wall or agent data
    m_pWorld: Rc<RefCell<GameWorld>>,

    //the steering behavior class
    m_pSteering: Option<Rc<RefCell<SteeringBehavior>>>,

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

    moving_entity: MovingEntity,
}

impl Vehicle {
    pub fn new(
        world: Rc<RefCell<GameWorld>>,
        position: Vec2,
        rotation: f32,
        velocity: Vec2,
        mass: f32,
        max_force: f32,
        max_speed: f32,
        max_turn_rate: f32,
        scale: f32,
    ) -> Rc<RefCell<Vehicle>> {
        let moving_entity = MovingEntity::new(
            position,
            scale,
            velocity,
            max_speed,
            vec2(rotation.sin(), -rotation.cos()),
            mass,
            vec2(scale, scale),
            max_turn_rate,
            max_force
        );

        let heading_smoother = Smoother::new(PRM.NumSamplesForSmoothing, vec2(0.0, 0.0));

        let vehicle = Rc::new(RefCell::new(Vehicle {
            m_pWorld: world,
            m_pSteering: None,
            m_pHeadingSmoother: heading_smoother,
            m_vSmoothedHeading: Default::default(),
            m_bSmoothingOn: false,
            m_dTimeElapsed: 0.0,
            m_vecVehicleVB: vec![],
            moving_entity,
        }));

        let steering = Rc::new(RefCell::new(SteeringBehavior::new(vehicle.clone())));
        vehicle.borrow_mut().m_pSteering = Some(steering);

        vehicle
    }

    pub fn Steering(&self) -> Rc<RefCell<SteeringBehavior>> {
        if let Some(steering) = &self.m_pSteering {
           steering.clone()
        } else {
            panic!("m_pSteering has not been initialized.")
        }
    }
}

impl EntityBase for Vehicle {
    fn Pos(&self) -> Vec2 {
        self.moving_entity.base_entity.m_vPos.clone()
    }

    fn Tag(&mut self) {
        self.moving_entity.base_entity.Tag();
    }

    fn UnTag(&mut self) {
        self.moving_entity.base_entity.UnTag();
    }

    fn Scale(&self) -> Vec2 {
        self.moving_entity.base_entity.m_vScale.clone()
    }

    fn SetScale_vec(&mut self, val: Vec2) {
        self.moving_entity.base_entity.SetScale_vec(val);
    }

    fn SetScale_float(&mut self, val: f32) {
        self.moving_entity.base_entity.SetScale_float(val);
    }

    fn EntityType(&self) -> i32 {
        self.moving_entity.base_entity.EntityType()
    }

    fn SetEntityType(&mut self, new_type: i32) {
        self.moving_entity.base_entity.SetEntityType(new_type);
    }
}
