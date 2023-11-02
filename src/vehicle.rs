use crate::base_entity::{BaseEntity, EntityBase};
use crate::config_loader::CONFIG;
use crate::core::sprite_model::SpriteModel;
use crate::game_world::GameWorld;
use crate::moving_entity::MovingEntity;
use crate::smoother::Smoother;
use crate::steering_behavior::SteeringBehavior;
use crate::utils::{RandInRange, Truncate, WrapAround};
use glam::{vec2, vec3, Vec2, Vec3};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Vehicle {
    pub base_entity: BaseEntity,

    pub moving_entity: MovingEntity,

    //a pointer to the world data. So a vehicle can access any obstacle,
    //path, wall or agent data
    pub m_pWorld: Rc<RefCell<GameWorld>>,

    //the steering behavior class
    pub m_pSteering: RefCell<SteeringBehavior>,

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
    pub(crate) m_dTimeElapsed: f32,

    //buffer for the vehicle shape
    m_vecVehicleVB: Vec<Vec2>,

    color: Vec3,

    sprite_model: SpriteModel,
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
        sprite: SpriteModel,
    ) -> Rc<RefCell<Vehicle>> {
        let mut base_entity = BaseEntity::with_type_and_position(0, position, scale);
        base_entity.scale = vec2(scale, scale);

        let moving_entity = MovingEntity::new(
            velocity,
            max_speed,
            vec2(rotation.sin(), -rotation.cos()),
            mass,
            max_turn_rate,
            max_force,
        );

        let heading_smoother = Smoother::new(CONFIG.NumSamplesForSmoothing, vec2(0.0, 0.0));

        let color = vec3(RandInRange(0.2, 1.0), RandInRange(0.2, 1.0), RandInRange(0.2, 1.0));

        let vehicle = Rc::new(RefCell::new(Vehicle {
            base_entity,
            m_pWorld: world,
            m_pSteering: RefCell::new(SteeringBehavior::new()),
            m_pHeadingSmoother: heading_smoother,
            m_vSmoothedHeading: Default::default(),
            m_bSmoothingOn: false,
            m_dTimeElapsed: 0.0,
            m_vecVehicleVB: vec![],
            moving_entity,
            color,
            sprite_model: sprite,
        }));

        let id = vehicle.borrow().id();
        if id == 0 {
            vehicle.borrow().m_pSteering.borrow_mut().m_vWanderTarget = vec2(0.0, 0.0);
        }

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
    pub fn Update(vehicle: &Rc<RefCell<Vehicle>>, time_elapsed: f32) -> Vec2 {
        // update the time elapsed
        vehicle.borrow_mut().m_dTimeElapsed = time_elapsed;

        // keep a record of its old position so we can update its cell later in this method
        let old_pos = vehicle.borrow().position();

        // calculate the combined force from each steering behavior in the vehicle's list
        let steering_force = vehicle.borrow().m_pSteering.borrow_mut().Calculate(vehicle);

        // Acceleration = Force/Mass
        let acceleration = steering_force / vehicle.borrow().moving_entity.mass;

        // update velocity
        vehicle.borrow_mut().moving_entity.velocity += acceleration * time_elapsed;

        // make sure vehicle does not exceed maximum velocity
        // vehicle.moving_entity.m_vVelocity.Truncate(vehicle.moving_entity.m_dMaxSpeed);
        let velocity = vehicle.borrow().moving_entity.velocity;
        let max_speed = vehicle.borrow().moving_entity.max_speed;
        let truncated_velocity = Truncate(velocity, max_speed);
        vehicle.borrow_mut().moving_entity.velocity = truncated_velocity;

        // update the position
        let travel_distance = vehicle.borrow().moving_entity.velocity * time_elapsed;
        vehicle.borrow_mut().base_entity.position += travel_distance;

        // update the heading if the vehicle has a non zero velocity
        if vehicle.borrow().moving_entity.velocity.length_squared() > 0.00000001 {
            let normalize = vehicle.borrow().moving_entity.velocity.normalize_or_zero();
            vehicle.borrow_mut().moving_entity.heading = normalize;

            let prep = vehicle.borrow().moving_entity.heading.perp();
            vehicle.borrow_mut().moving_entity.side_vec = prep;
        }

        //EnforceNonPenetrationConstraint(this, World()->Agents());

        //treat the screen as a toroid
        let cx = vehicle.borrow().m_pWorld.borrow().cxClient();
        let cy = vehicle.borrow().m_pWorld.borrow().cyClient();
        WrapAround(&mut vehicle.borrow_mut().base_entity.position, cx, cy);

        // TODO: Note, this moved this to gameworld object
        //update the vehicle's current cell if space partitioning is turned on
        // if vehicle.Steering().isSpacePartitioningOn() {
        //     vehicle.m_pWorld.borrow_mut().m_pCellSpace.UpdateEntity(this, &OldPos);
        // }

        if vehicle.borrow().m_bSmoothingOn {
            let heading = vehicle.borrow().moving_entity.heading;
            let smoothed_heading = vehicle.borrow_mut().m_pHeadingSmoother.update(heading);
            vehicle.borrow_mut().m_vSmoothedHeading = smoothed_heading;
        }

        old_pos
    }

    pub fn render(&mut self, delta_time: f32) {
        let mut angle = self.moving_entity.heading.x.acos().to_degrees();

        if self.moving_entity.heading.y < 0.0 {
            angle = 360.0 - angle;
        }

        let position = vec3(self.base_entity.position.x, self.base_entity.position.y, 0.0);
        let scale = vec3(self.base_entity.scale.x, self.base_entity.scale.y, 1.0);

        self.sprite_model.render(position, angle - 90.0, scale, delta_time);

        // println!("fish id: {}   position: {}", self.ID(), position);

        /*
            //a vector to hold the transformed vertices
            static std::vector<Vector2D>  m_vecVehicleVBTrans;
            static C2DMatrix m_transform;

            //render neighboring vehicles in different colors if requested
            if (m_pWorld->RenderNeighbors())
            {
                if (ID() == 0)
                    gdi->RedPen();
                else if(IsTagged())
                    gdi->GreenPen();
                else
                    gdi->BluePen();
            }
            else
            {
                gdi->BluePen();
            }

            if (Steering()->isInterposeOn())
            {
                gdi->RedPen();
            }

            if (Steering()->isHideOn())
            {
                gdi->GreenPen();
            }

            if (isSmoothingOn())
            {

                m_vecVehicleVBTrans = WorldTransform(m_vecVehicleVB,
                                                     Pos(),
                                                     SmoothedHeading(),
                                                     SmoothedHeading().Perp(),
                                                     Scale());
            }
            else
            {
                m_vecVehicleVBTrans = WorldTransform(m_vecVehicleVB,
                                                     Pos(),
                                                     Heading(),
                                                     Side(),
                                                     Scale());
            }


            gdi->ClosedShape(m_vecVehicleVBTrans);

        */

        // if self.m_bSmoothingOn {
        // {
        //     gdi->Triangle(Pos(),
        //                   SmoothedHeading(),
        //                   SmoothedHeading().Perp(),
        //                   Scale());
        //
        // } else {
        //gdi->Triangle(Pos(),
        //			  Heading(),
        //			  Side(),
        //			  Scale());
        // self.Triangle();
        // }

        //render any visual aids / and or user options
        // if (m_pWorld->ViewKeys())
        // {
        //     Steering()->RenderAids();
        // }
    }

    pub fn set_max_speed(&mut self, speed: f32) {
        self.moving_entity.max_speed = speed;
    }

    pub fn heading(&self) -> Vec2 {
        self.moving_entity.heading
    }
    pub fn side(&self) -> Vec2 {
        self.moving_entity.side_vec
    }

    pub fn print(&self) {
        println!(
            "{:#?}\n{:#?}", // "{:#?}\n", // "{:#?}\n{:#?}\n{:#?}\n", // {:#?}\n{:#?}\n",
            self.moving_entity,
            // unsafe {self.m_pSteering.try_borrow_unguarded()},
            // self.m_pHeadingSmoother,
            // self.m_vSmoothedHeading,
            // self.m_bSmoothingOn,
            self.m_dTimeElapsed,
            // self.m_vecVehicleVB,
            // self.color
        );
    }
}

impl EntityBase for Vehicle {
    fn id(&self) -> i32 {
        self.base_entity.id
    }

    fn position(&self) -> Vec2 {
        self.base_entity.position
    }

    fn bounding_radius(&self) -> f32 {
        self.base_entity.bounding_radius
    }

    fn tag(&mut self) {
        self.base_entity.tag();
    }

    fn untag(&mut self) {
        self.base_entity.untag();
    }

    fn is_tagged(&self) -> bool {
        self.base_entity.is_tagged()
    }

    fn scale(&self) -> Vec2 {
        self.base_entity.scale
    }

    fn set_scale_vec(&mut self, val: Vec2) {
        self.base_entity.set_scale_vec(val);
    }

    fn set_scale_float(&mut self, val: f32) {
        self.base_entity.set_scale_float(val);
    }

    fn entity_type(&self) -> i32 {
        self.base_entity.entity_type()
    }

    fn set_entity_type(&mut self, new_type: i32) {
        self.base_entity.set_entity_type(new_type);
    }
}
