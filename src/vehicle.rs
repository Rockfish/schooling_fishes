use crate::configuration::CONFIG;
use small_gl_core::model::Model;
use crate::entity_traits::{next_valid_id, EntityBase, EntityMovable, EntitySteerable};
use crate::game_world::GameWorld;
use crate::smoother::Smoother;
use crate::steering_behavior::SteeringBehavior;
use crate::utils::{RandInRange, Truncate, WrapAround};
use glam::{vec2, vec3, Vec2};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Vehicle {
    // EntityBase
    pub id: i32,
    pub entity_type: i32,
    pub tag: bool,
    pub position: Vec2,
    pub scale: Vec2,
    pub bounding_radius: f32,

    // EntityMovable
    pub velocity: Vec2,
    pub heading: Vec2,
    pub side_vec: Vec2,
    pub mass: f32,
    pub max_speed: f32,
    pub max_force: f32,
    pub max_turn_rate: f32,

    // EntitySteerable
    pub m_pSteering: RefCell<SteeringBehavior>,
    m_pHeadingSmoother: Smoother<Vec2>,
    m_vSmoothedHeading: Vec2,
    m_bSmoothingOn: bool,

    // temp
    height: f32, // create some depth

    //keeps a track of the most recent update time. (some of the
    //steering behaviors make use of this - see Wander)
    pub m_dTimeElapsed: f32,

    pub m_pWorld: Rc<RefCell<GameWorld>>,

    //sprite_model: SpriteModel,
    model: Model,
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
        model: Model,
    ) -> Rc<RefCell<Vehicle>> {
        let heading = vec2(rotation.sin(), -rotation.cos());

        let heading_smoother = Smoother::new(CONFIG.NumSamplesForSmoothing, vec2(0.0, 0.0));

        let vehicle = Rc::new(RefCell::new(Vehicle {
            id: next_valid_id(),
            entity_type: 0,
            tag: false,
            position,
            scale: vec2(scale, scale),
            bounding_radius: 0.0,
            velocity,
            heading,
            side_vec: Default::default(),
            mass,
            max_speed,
            max_force,
            max_turn_rate,
            m_pWorld: world,
            m_pSteering: RefCell::new(SteeringBehavior::new()),
            m_pHeadingSmoother: heading_smoother,
            m_vSmoothedHeading: Default::default(),
            m_bSmoothingOn: true,
            m_dTimeElapsed: 0.0,
            model,
            height: RandInRange(0.0, 50.0),
        }));

        let id = vehicle.borrow().id();
        if id == 0 {
            vehicle.borrow().m_pSteering.borrow_mut().m_vWanderTarget = vec2(0.0, 0.0);
        }

        vehicle
    }

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
        let acceleration = steering_force / vehicle.borrow().mass;

        // update velocity
        vehicle.borrow_mut().velocity += acceleration * time_elapsed;

        // make sure vehicle does not exceed maximum velocity
        // vehicle.m_vVelocity.Truncate(vehicle.m_dMaxSpeed);
        let velocity = vehicle.borrow().velocity;
        let max_speed = vehicle.borrow().max_speed;
        let truncated_velocity = Truncate(velocity, max_speed);
        vehicle.borrow_mut().velocity = truncated_velocity;

        // update the position
        let travel_distance = vehicle.borrow().velocity * time_elapsed;
        vehicle.borrow_mut().position += travel_distance;

        // update the heading if the vehicle has a non zero velocity
        if vehicle.borrow().velocity.length_squared() > 0.00000001 {
            let normalize = vehicle.borrow().velocity.normalize_or_zero();
            vehicle.borrow_mut().heading = normalize;

            let prep = vehicle.borrow().heading.perp();
            vehicle.borrow_mut().side_vec = prep;
        }

        //EnforceNonPenetrationConstraint(this, World()->Agents());

        //treat the screen as a toroid
        let cx = vehicle.borrow().m_pWorld.borrow().cxClient();
        let cy = vehicle.borrow().m_pWorld.borrow().cyClient();
        WrapAround(&mut vehicle.borrow_mut().position, cx, cy);

        if vehicle.borrow().m_bSmoothingOn {
            let heading = vehicle.borrow().heading;
            let smoothed_heading = vehicle.borrow_mut().m_pHeadingSmoother.update(heading);
            vehicle.borrow_mut().m_vSmoothedHeading = smoothed_heading;
        }

        old_pos
    }

    pub fn SmoothedHeading(&self) -> Vec2 {
        self.m_vSmoothedHeading
    }

    pub fn render(&mut self, delta_time: f32) {
        let mut angle = 0.0f32;

        if self.m_bSmoothingOn {
            angle = self.m_vSmoothedHeading.x.acos().to_degrees();
            if self.m_vSmoothedHeading.y < 0.0 {
                angle = 360.0 - angle;
            }
        } else {
            angle = self.heading.x.acos().to_degrees();
            if self.heading.y < 0.0 {
                angle = 360.0 - angle;
            }
        }

        // fix model orientation
        angle += 90.0;
        angle *= -1.0;

        // let position = vec3(self.position.x, self.position.y, 0.0);
        let position = vec3(self.position.x - 400.0, self.height, self.position.y - 400.0);
        let scale = vec3(self.scale.x, self.scale.y, self.scale.x);

        self.model.render(position, angle, scale, delta_time);

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
}

impl EntityBase for Vehicle {
    fn id(&self) -> i32 {
        self.id
    }
    fn entity_type(&self) -> i32 {
        self.entity_type
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn bounding_radius(&self) -> f32 {
        self.bounding_radius
    }

    fn tag(&mut self) {
        self.tag = true;
    }

    fn untag(&mut self) {
        self.tag = false;
    }

    fn is_tagged(&self) -> bool {
        self.tag
    }

    fn scale(&self) -> Vec2 {
        self.scale
    }

    fn set_scale_vec(&mut self, val: Vec2) {
        self.scale = val;
    }

    fn set_scale_float(&mut self, val: f32) {
        self.scale = vec2(val, val);
    }
}

impl EntityMovable for Vehicle {
    fn mass(&self) -> f32 {
        self.mass
    }

    fn velocity(&self) -> Vec2 {
        self.velocity
    }

    fn speed(&self) -> f32 {
        self.velocity.length()
    }

    fn heading(&self) -> Vec2 {
        self.heading
    }

    fn side(&self) -> Vec2 {
        self.side_vec
    }

    fn max_force(&self) -> f32 {
        self.max_force
    }

    fn max_speed(&self) -> f32 {
        self.max_speed
    }

    fn set_max_speed(&mut self, speed: f32) {
        self.max_speed = speed;
    }
}

impl EntitySteerable for Vehicle { }
