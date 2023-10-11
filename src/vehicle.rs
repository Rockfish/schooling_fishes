use std::cell::RefCell;
use std::rc::Rc;
use crate::game_world::GameWorld;
use crate::smoother::Smoother;
use crate::steering_behavior::SteeringBehavior;
use glam::{Vec2, vec2};
use crate::base_entity::EntityBase;
use crate::moving_entity::MovingEntity;
use crate::param_loader::PRM;
use crate::utils::{Truncate, WrapAround};

pub struct Vehicle {
    //a pointer to the world data. So a vehicle can access any obstacle,
    //path, wall or agent data
    pub m_pWorld: Rc<RefCell<GameWorld>>,

    //the steering behavior class
    pub m_pSteering: Option<SteeringBehavior>,

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

    #[allow(clippy::too_many_arguments)]
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
            max_force,
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

        let steering = SteeringBehavior::new(vehicle.clone());
        vehicle.borrow_mut().m_pSteering = Some(steering);

        vehicle
    }

    // pub fn Steering(&self) -> Rc<RefCell<SteeringBehavior>> {
    //     if let Some(steering) = &self.m_pSteering {
    //        steering.clone()
    //     } else {
    //         panic!("m_pSteering has not been initialized.")
    //     }
    // }

    //------------------------------ Update ----------------------------------
    //
    //  Updates the vehicle's position from a series of steering behaviors
    //------------------------------------------------------------------------
    pub fn Update(rc_vehicle: &Rc<RefCell<Vehicle>>, time_elapsed: f32) -> Vec2 {
        let mut vehicle = rc_vehicle.borrow_mut();
        //update the time elapsed
        vehicle.m_dTimeElapsed = time_elapsed;

        //keep a record of its old position so we can update its cell later
        //in this method
        let OldPos = vehicle.Pos();

        //calculate the combined force from each steering behavior in the
        //vehicle's list
        let SteeringForce = vehicle.m_pSteering.as_mut().unwrap().Calculate();

        //Acceleration = Force/Mass
        let acceleration = SteeringForce / vehicle.moving_entity.m_dMass;

        //update velocity
        vehicle.moving_entity.m_vVelocity += acceleration * time_elapsed;

        //make sure vehicle does not exceed maximum velocity
        // vehicle.moving_entity.m_vVelocity.Truncate(vehicle.moving_entity.m_dMaxSpeed);
        vehicle.moving_entity.m_vVelocity = Truncate(vehicle.moving_entity.m_vVelocity, vehicle.moving_entity.m_dMaxSpeed);

        //update the position
        let velo = vehicle.moving_entity.m_vVelocity * time_elapsed;
        vehicle.moving_entity.base_entity.m_vPos += velo;
        // vehicle.moving_entity.base_entity.m_vPos += vehicle.moving_entity.m_vVelocity.clone() * time_elapsed;

        //update the heading if the vehicle has a non zero velocity
        if vehicle.moving_entity.m_vVelocity.length_squared() > 0.00000001 {
            vehicle.moving_entity.m_vHeading = vehicle.moving_entity.m_vVelocity.normalize();
            vehicle.moving_entity.m_vSide = vehicle.moving_entity.m_vHeading.perp();
        }

        //EnforceNonPenetrationConstraint(this, World()->Agents());

        //treat the screen as a toroid
        let cx = vehicle.m_pWorld.borrow().cxClient();
        let cy = vehicle.m_pWorld.borrow().cyClient();
        WrapAround(&mut vehicle.moving_entity.base_entity.m_vPos, cx, cy);

        // TODO: Note, this moved this to gameworld object
        //update the vehicle's current cell if space partitioning is turned on
        // if vehicle.Steering().isSpacePartitioningOn() {
        //     vehicle.m_pWorld.borrow_mut().m_pCellSpace.UpdateEntity(this, &OldPos);
        // }

        if vehicle.m_bSmoothingOn {
            let heading = vehicle.moving_entity.m_vHeading;
            vehicle.m_vSmoothedHeading = vehicle.m_pHeadingSmoother.update(heading);
        }
        OldPos
    }

    /*-------------------------------------------accessor methods
    // for reference only since accessors are more of a cpp pattern than rust

    SteeringBehavior*const Steering(&self)const {return m_pSteering;}
    GameWorld*const World()const {return m_pWorld;}
    Vector2D SmoothedHeading()const {return m_vSmoothedHeading;}
    bool isSmoothingOn()const {return m_bSmoothingOn;}
    void SmoothingOn() {m_bSmoothingOn = true;}
    void SmoothingOff() {m_bSmoothingOn = false;}
    void ToggleSmoothing() {m_bSmoothingOn = !m_bSmoothingOn;}

    float TimeElapsed()const {return m_dTimeElapsed;}

     */
}

impl EntityBase for Vehicle {
    fn ID(&self) -> i32 {
        self.moving_entity.base_entity.m_ID
    }

    fn Pos(&self) -> Vec2 {
        self.moving_entity.base_entity.m_vPos
    }

    fn BRadius(&self) -> f32 {
        self.moving_entity.base_entity.m_dBoundingRadius
    }

    fn Tag(&mut self) {
        self.moving_entity.base_entity.Tag();
    }

    fn UnTag(&mut self) {
        self.moving_entity.base_entity.UnTag();
    }

    fn Scale(&self) -> Vec2 {
        self.moving_entity.base_entity.m_vScale
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
