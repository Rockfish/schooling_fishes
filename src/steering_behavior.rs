//--------------------------- Constants ----------------------------------

use crate::configuration::CONFIG;
use crate::entity_traits::{EntityBase, EntityMovable, EntitySteerable};
use crate::path::Path;
use crate::transformations::PointToWorldSpace;
use crate::utils::{min, RandFloat, RandInRange, RandomClamped};
use crate::vehicle::Vehicle;
use crate::wall_2d::Wall2D;
use glam::{vec2, Vec2};
use std::cell::{Ref, RefCell};
use std::f32::consts::TAU;
use std::ops::Div;
use std::rc::Rc;

//the radius of the constraining circle for the wander behavior
const WANDER_RAD: f32 = 1.2;
//distance the wander circle is projected in front of the agent
const WANDER_DIST: f32 = 2.0;
//the maximum amount of displacement along the circle each frame
const WANDER_JITTER_PER_SEC: f32 = 80.0;
//used in path following
const WAYPOINT_SEEK_DIST: f32 = 20.0;

//------------------------------------------------------------------------
#[derive(Debug)]
pub enum Deceleration {
    slow = 3,
    normal = 2,
    fast = 1,
}

#[derive(Debug)]
pub enum SummingMethod {
    weighted_average,
    prioritized,
    dithered,
}

#[derive(Debug, Copy, Clone)]
pub enum BehaviorType {
    none = 0x00000,
    seek = 0x00002,
    flee = 0x00004,
    arrive = 0x00008,
    wander = 0x00010,
    cohesion = 0x00020,
    separation = 0x00040,
    alignment = 0x00080,
    obstacle_avoidance = 0x00100,
    wall_avoidance = 0x00200,
    follow_path = 0x00400,
    pursuit = 0x00800,
    evade = 0x01000,
    interpose = 0x02000,
    hide = 0x04000,
    flock = 0x08000,
    offset_pursuit = 0x10000,
}

pub struct SteeringBehavior {
    // the steering force created by the combined effect of all the selected behaviors
    pub m_vSteeringForce: Vec2,

    // these can be used to keep track of friends, pursuers, or prey
    m_pTargetAgent1: Option<Rc<RefCell<Vehicle>>>,
    m_pTargetAgent2: Option<Rc<RefCell<Vehicle>>>,

    // the current target
    pub m_vTarget: Vec2,

    // length of the 'detection box' utilized in obstacle avoidance
    m_dDBoxLength: f32,

    // a vertex buffer to contain the feelers rqd for wall avoidance
    m_Feelers: Vec<Vec2>,

    // the length of the 'feeler/s' used in wall detection
    m_dWallDetectionFeelerLength: f32,

    // the current position on the wander circle the agent is
    // attempting to steer towards
    pub m_vWanderTarget: Vec2,

    // explained above
    m_dWanderJitter: f32,
    m_dWanderRadius: f32,
    m_dWanderDistance: f32,
    wander_direction_time: f32,

    // multipliers. These can be adjusted to effect strength of the
    //  appropriate behavior. Useful to get flocking the way you require
    // for example.
    m_dWeightSeparation: f32,
    m_dWeightCohesion: f32,
    m_dWeightAlignment: f32,
    m_dWeightWander: f32,
    m_dWeightObstacleAvoidance: f32,
    m_dWeightWallAvoidance: f32,
    m_dWeightSeek: f32,
    m_dWeightFlee: f32,
    m_dWeightArrive: f32,
    m_dWeightPursuit: f32,
    m_dWeightOffsetPursuit: f32,
    m_dWeightInterpose: f32,
    m_dWeightHide: f32,
    m_dWeightEvade: f32,
    m_dWeightFollowPath: f32,

    // how far the agent can 'see'
    m_dViewDistance: f32,

    // pointer to any current path
    m_pPath: Path,

    // the distance (squared) a vehicle has to be from a path waypoint before
    // it starts seeking to the next waypoint
    m_dWaypointSeekDistSq: f32,

    // any offset used for formations or offset pursuit
    m_vOffset: Vec2,

    // binary flags to indicate whether or not a behavior should be active
    m_iFlags: i32,

    // Arrive makes use of these to determine how quickly a vehicle
    // should decelerate to its target
    // default
    m_Deceleration: Deceleration,

    // is cell space partitioning to be used or not?
    pub m_bCellSpaceOn: bool,

    // what type of method is used to sum any active behavior
    m_SummingMethod: SummingMethod,
}

impl SteeringBehavior {
    pub fn new() -> Self {
        let wander_radius = WANDER_RAD;
        let theta = RandFloat() * TAU;
        let wander_target = vec2(wander_radius * theta.cos(), wander_radius * theta.sin());

        let mut path = Path::default();
        path.LoopOn();

        SteeringBehavior {
            m_iFlags: 0,
            m_dDBoxLength: CONFIG.MinDetectionBoxLength,
            m_dWeightCohesion: CONFIG.CohesionWeight,
            m_dWeightAlignment: CONFIG.AlignmentWeight,
            m_dWeightSeparation: CONFIG.SeparationWeight,
            m_dWeightObstacleAvoidance: CONFIG.ObstacleAvoidanceWeight,
            m_dWeightWander: CONFIG.WanderWeight,
            m_dWeightWallAvoidance: CONFIG.WallAvoidanceWeight,
            m_dViewDistance: CONFIG.ViewDistance,
            m_dWallDetectionFeelerLength: CONFIG.WallDetectionFeelerLength,
            m_Feelers: vec![], // 3, ?
            m_Deceleration: Deceleration::normal,
            m_pTargetAgent1: None,
            m_pTargetAgent2: None,
            m_dWanderDistance: WANDER_DIST,
            m_dWanderJitter: WANDER_JITTER_PER_SEC,
            m_dWanderRadius: wander_radius,
            wander_direction_time: 0.0,
            m_dWaypointSeekDistSq: WAYPOINT_SEEK_DIST * WAYPOINT_SEEK_DIST,
            m_dWeightSeek: CONFIG.SeekWeight,
            m_dWeightFlee: CONFIG.FleeWeight,
            m_dWeightArrive: CONFIG.ArriveWeight,
            m_dWeightPursuit: CONFIG.PursuitWeight,
            m_dWeightOffsetPursuit: CONFIG.OffsetPursuitWeight,
            m_dWeightInterpose: CONFIG.InterposeWeight,
            m_dWeightHide: CONFIG.HideWeight,
            m_dWeightEvade: CONFIG.EvadeWeight,
            m_dWeightFollowPath: CONFIG.FollowPathWeight,
            m_bCellSpaceOn: false,
            m_SummingMethod: SummingMethod::prioritized,
            m_vWanderTarget: wander_target,
            m_pPath: path,
            m_vSteeringForce: Default::default(),
            m_vTarget: Default::default(),
            m_vOffset: Default::default(),
        }
    }

    pub fn FlockingOn(&mut self) {
        self.CohesionOn();
        self.AlignmentOn();
        self.SeparationOn();
        self.WanderOn();
    }

    pub fn FlockingOff(&mut self) {
        self.CohesionOff();
        self.AlignmentOff();
        self.SeparationOff();
        self.WanderOff();
    }

    /*
    void FleeOn(){m_iFlags |= flee;}
    void SeekOn(){m_iFlags |= seek;}
    void ArriveOn(){m_iFlags |= arrive;}
     */

    pub fn WanderOn(&mut self) {
        self.m_iFlags |= BehaviorType::wander as i32;
    }
    pub fn PursuitOn(&mut self, target: Rc<RefCell<Vehicle>>) {
        self.m_iFlags |= BehaviorType::pursuit as i32;
        self.m_pTargetAgent1 = Some(target);
    }

    pub fn EvadeOn(&mut self, target: Rc<RefCell<Vehicle>>) {
        self.m_iFlags |= BehaviorType::evade as i32;
        self.m_pTargetAgent1 = Some(target);
    }

    pub fn CohesionOn(&mut self) {
        self.m_iFlags |= BehaviorType::cohesion as i32;
    }
    pub fn SeparationOn(&mut self) {
        self.m_iFlags |= BehaviorType::separation as i32;
    }
    pub fn AlignmentOn(&mut self) {
        self.m_iFlags |= BehaviorType::alignment as i32;
    }
    pub fn CohesionOff(&mut self) {
        self.m_iFlags ^= BehaviorType::cohesion as i32;
    }
    pub fn SeparationOff(&mut self) {
        self.m_iFlags ^= BehaviorType::separation as i32;
    }
    pub fn AlignmentOff(&mut self) {
        self.m_iFlags ^= BehaviorType::alignment as i32;
    }
    pub fn WanderOff(&mut self) {
        self.m_iFlags ^= BehaviorType::wander as i32;
    }

    pub fn isSpacePartitioningOn(&self) -> bool {
        self.m_bCellSpaceOn
    }

    //this function tests if a specific bit of m_iFlags is set
    pub fn On(&self, bt: BehaviorType) -> bool {
        (self.m_iFlags & bt as i32) == bt as i32
    }

    pub fn Calculate(&mut self, vehicle: &Rc<RefCell<Vehicle>>) -> Vec2 {
        // reset the steering force
        self.m_vSteeringForce.x = 0.0;
        self.m_vSteeringForce.y = 0.0;

        if !self.m_bCellSpaceOn {
            if self.On(BehaviorType::separation) || self.On(BehaviorType::alignment) || self.On(BehaviorType::cohesion) {
                let world = vehicle.borrow().m_pWorld.clone();

                world.borrow().TagVehiclesWithinViewRange(vehicle.clone(), self.m_dViewDistance);
            }
        } else {
            // calculate neighbours in cell-space if any of the following 3 group
            // behaviors are switched on
            if self.On(BehaviorType::separation) || self.On(BehaviorType::alignment) || self.On(BehaviorType::cohesion) {
                let position = vehicle.borrow().position();

                vehicle
                    .borrow()
                    .m_pWorld
                    .borrow()
                    .m_pCellSpace
                    .borrow_mut()
                    .CalculateNeighbors(position, self.m_dViewDistance);
            }
        }

        let new_steering_force = match self.m_SummingMethod {
            SummingMethod::weighted_average => self.CalculateWeightedSum(),
            SummingMethod::prioritized => self.CalculatePrioritized(vehicle),
            SummingMethod::dithered => self.CalculateDithered(),
        };

        self.m_vSteeringForce = new_steering_force;

        self.m_vSteeringForce
    }

    //--------------------- AccumulateForce ----------------------------------
    //
    //  This function calculates how much of its max steering force the
    //  vehicle has left to apply and then applies that amount of the
    //  force to add.
    //------------------------------------------------------------------------
    pub fn AccumulateForce(vehicle: &Rc<RefCell<Vehicle>>, running_total: &mut Vec2, force_to_add: Vec2) -> bool {
        // calculate how much steering force the vehicle has used so far
        let magnitude_so_far = running_total.length();

        // calculate how much steering force remains to be used by this vehicle
        let magnitude_remaining = vehicle.borrow().max_force() - magnitude_so_far;

        // return false if there is no more force left to use
        if magnitude_remaining <= 0.0 {
            return false;
        }

        // calculate the magnitude of the force we want to add
        let magnitude_to_add = force_to_add.length();

        // if the magnitude of the sum of ForceToAdd and the running total
        // does not exceed the maximum force available to this vehicle, just
        // add together. Otherwise add as much of the ForceToAdd vector is
        // possible without going over the max.
        if magnitude_to_add < magnitude_remaining {
            *running_total += force_to_add;
        } else {
            // add it to the steering force
            *running_total += force_to_add.normalize_or_zero() * magnitude_remaining;
        }

        return true;
    }

    pub fn CalculateWeightedSum(&mut self) -> Vec2 {
        todo!()
    }

    //---------------------- CalculatePrioritized ----------------------------
    //
    //  this method calls each active steering behavior in order of priority
    //  and accumulates their forces until the max steering force magnitude
    //  is reached, at which time the function returns the steering force
    //  accumulated to that  point
    //------------------------------------------------------------------------
    pub fn CalculatePrioritized(&mut self, vehicle: &Rc<RefCell<Vehicle>>) -> Vec2 {
        let mut force: Vec2 = Vec2::default();
        /*
            if (On(wall_avoidance))
            {
                force = WallAvoidance(vehicle->World()->Walls()) *
                m_dWeightWallAvoidance;

                if (!SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force)) return self.m_vSteeringForce;
            }
            }

            if (On(obstacle_avoidance))
            {
                force = ObstacleAvoidance(vehicle->World()->Obstacles()) *
                m_dWeightObstacleAvoidance;

                if (!SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force)) return self.m_vSteeringForce;
            }
            }
        */
        if self.On(BehaviorType::evade) {
            assert!(&self.m_pTargetAgent1.is_some(), "Evade target not assigned");

            force = SteeringBehavior::Evade(vehicle, self.m_pTargetAgent1.as_mut().unwrap().borrow()) * self.m_dWeightEvade;

            if !SteeringBehavior::AccumulateForce(vehicle, &mut self.m_vSteeringForce, force) {
                return self.m_vSteeringForce;
            }
        }

        if self.On(BehaviorType::flee) {
            force = SteeringBehavior::Flee(vehicle, vehicle.borrow().m_pWorld.borrow().m_vCrosshair) * self.m_dWeightFlee;

            if !SteeringBehavior::AccumulateForce(vehicle, &mut self.m_vSteeringForce, force) {
                return self.m_vSteeringForce;
            }
        }

        // these next three can be combined for flocking behavior (wander is
        // also a good behavior to add into this mix)
        if !self.isSpacePartitioningOn() {
            // if self.On(BehaviorType::separation)
            // {
            //     force = Separation(vehicle->World()->Agents()) * m_dWeightSeparation;
            //
            //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
            //         return self.m_vSteeringForce;
            //     }
            // }

            // if self.On(BehaviorType::alignment)
            // {
            //     force = Alignment(vehicle->World()->Agents()) * m_dWeightAlignment;
            //
            //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
            //         return self.m_vSteeringForce;
            //     }
            // }

            if self.On(BehaviorType::cohesion) {
                force = SteeringBehavior::Cohesion(vehicle, &self.m_pTargetAgent1, &vehicle.borrow().m_pWorld.borrow().m_Vehicles)
                    * self.m_dWeightCohesion;

                if !SteeringBehavior::AccumulateForce(vehicle, &mut self.m_vSteeringForce, force) {
                    return self.m_vSteeringForce;
                }
            }
        } else {
            if self.On(BehaviorType::separation) {
                force = SteeringBehavior::SeparationPlus(vehicle, &vehicle.borrow().m_pWorld.borrow().m_pCellSpace.borrow().m_Neighbors)
                    * self.m_dWeightSeparation;

                if !SteeringBehavior::AccumulateForce(vehicle, &mut self.m_vSteeringForce, force) {
                    return self.m_vSteeringForce;
                }
            }

            if self.On(BehaviorType::alignment) {
                force = SteeringBehavior::AlignmentPlus(vehicle, &vehicle.borrow().m_pWorld.borrow().m_pCellSpace.borrow().m_Neighbors)
                    * self.m_dWeightAlignment;

                if !SteeringBehavior::AccumulateForce(vehicle, &mut self.m_vSteeringForce, force) {
                    return self.m_vSteeringForce;
                }
            }

            if self.On(BehaviorType::cohesion) {
                force = SteeringBehavior::CohesionPlus(vehicle, &vehicle.borrow().m_pWorld.borrow().m_pCellSpace.borrow().m_Neighbors)
                    * self.m_dWeightCohesion;

                if !SteeringBehavior::AccumulateForce(vehicle, &mut self.m_vSteeringForce, force) {
                    return self.m_vSteeringForce;
                }
            }
        }

        // if self.On(BehaviorType::seek)
        // {
        //     force = SteeringBehavior::Seek(vehicle.World().Crosshair()) * m_dWeightSeek;
        //
        //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
        //         return self.m_vSteeringForce;
        //     }
        // }
        //
        //
        // if self.On(BehaviorType::arrive)
        // {
        //     force = self.Arrive(vehicle.World().Crosshair(), m_Deceleration) * m_dWeightArrive;
        //
        //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
        //         return self.m_vSteeringForce;
        //     }
        // }

        if self.On(BehaviorType::wander) {
            force = self.Wander(vehicle) * self.m_dWeightWander;

            if !SteeringBehavior::AccumulateForce(vehicle, &mut self.m_vSteeringForce, force) {
                return self.m_vSteeringForce;
            }
        }

        // if self.On(BehaviorType::pursuit)
        // {
        //     assert(m_pTargetAgent1 && "pursuit target not assigned");
        //
        //     force = Pursuit(m_pTargetAgent1) * m_dWeightPursuit;
        //
        //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
        //         return self.m_vSteeringForce;
        //     }
        // }
        //
        // if self.On(BehaviorType::offset_pursuit)
        // {
        //     assert!(m_pTargetAgent1 && "pursuit target not assigned");
        //     assert!(!m_vOffset.isZero() && "No offset assigned");
        //
        //     force = self.OffsetPursuit(m_pTargetAgent1, m_vOffset);
        //
        //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
        //         return self.m_vSteeringForce;
        //     }
        // }
        //
        // if self.On(BehaviorType::interpose)
        // {
        //     assert!(m_pTargetAgent1 && m_pTargetAgent2, "Interpose agents not assigned");
        //
        //     force = Interpose(m_pTargetAgent1, m_pTargetAgent2) * m_dWeightInterpose;
        //
        //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
        //         return self.m_vSteeringForce;
        //     }
        // }
        //
        // if self.On(BehaviorType::hide)
        // {
        //     assert!(m_pTargetAgent1 && "Hide target not assigned");
        //
        //     force = self.Hide(m_pTargetAgent1, vehicle.World().Obstacles()) * self.m_dWeightHide;
        //
        //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
        //         return self.m_vSteeringForce;
        //     }
        // }
        //
        //
        // if self.On(BehaviorType::follow_path)
        // {
        //     force = FollowPath() * self.m_dWeightFollowPath;
        //
        //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
        //         return self.m_vSteeringForce;
        //     }
        // }

        return self.m_vSteeringForce;
    }

    pub fn CalculateDithered(&mut self) -> Vec2 {
        todo!()
    }

    /////////////////////////////////////////////////////////////////////////////// START OF BEHAVIORS

    //------------------------------- Seek -----------------------------------
    //
    //  Given a target, this behavior returns a steering force which will
    //  direct the agent towards the target
    //------------------------------------------------------------------------
    pub fn Seek(vehicle: &Rc<RefCell<Vehicle>>, TargetPos: Vec2) -> Vec2 {
        let mut desired_velocity = TargetPos - vehicle.borrow().position();
        desired_velocity = desired_velocity.normalize_or_zero();
        desired_velocity *= vehicle.borrow().max_speed();

        desired_velocity - vehicle.borrow().velocity()
    }

    //----------------------------- Flee -------------------------------------
    //
    //  Does the opposite of Seek
    //------------------------------------------------------------------------
    pub fn Flee(vehicle: &Rc<RefCell<Vehicle>>, TargetPos: Vec2) -> Vec2 {
        //only flee if the target is within 'panic distance'. Work in distance
        //squared space.
        /* const float PanicDistanceSq = 100.0f * 100.0;
        if (Vec2DDistanceSq(vehicle.Pos(), target) > PanicDistanceSq)
        {
        return Vec2(0,0);
        }
        */

        let mut desired_velocity = vehicle.borrow().position() - TargetPos;
        desired_velocity = desired_velocity.normalize_or_zero();
        desired_velocity *= vehicle.borrow().max_speed();

        return desired_velocity - vehicle.borrow().velocity();
    }

    //--------------------------- Arrive -------------------------------------
    //
    //  This behavior is similar to seek but it attempts to arrive at the
    //  target with a zero velocity
    //------------------------------------------------------------------------
    pub fn Arrive(vehicle: Vehicle, TargetPos: Vec2, deceleration: Deceleration) -> Vec2 {
        let ToTarget = TargetPos - vehicle.position();

        // calculate the distance to the target
        let dist = ToTarget.length();

        if dist > 0.0 {
            // because Deceleration is enumerated as an int, this value is required
            // to provide fine tweaking of the deceleration..
            let DecelerationTweaker: f32 = 0.3;

            // calculate the speed required to reach the target given the desired
            // deceleration
            let mut speed: f32 = dist / ((deceleration as i32) as f32 * DecelerationTweaker);

            // make sure the velocity does not exceed the max
            speed = min(speed, vehicle.max_speed());

            // from here proceed just like Seek except we don't need to normalize
            // the ToTarget vector because we have already gone to the trouble
            // of calculating its length: dist.
            let desired_velocity = ToTarget * speed / dist;

            return desired_velocity - vehicle.velocity();
        }

        vec2(0.0, 0.0)
    }

    //------------------------------ Pursuit ---------------------------------
    //
    //  this behavior creates a force that steers the agent towards the
    //  evader
    //------------------------------------------------------------------------
    pub fn Pursuit(&self, vehicle: &Rc<RefCell<Vehicle>>, evader: Vehicle) -> Vec2 {
        // if the evader is ahead and facing the agent then we can just seek
        // for the evader's current position.
        let ToEvader = evader.position() - vehicle.borrow().position();

        let RelativeHeading = vehicle.borrow().heading().dot(evader.heading());

        if (ToEvader.dot(vehicle.borrow().heading()) > 0.0) & &(RelativeHeading < -0.95)
        //acos(0.95)=18 degs
        {
            return SteeringBehavior::Seek(vehicle, evader.position());
        }

        // Not considered ahead so we predict where the evader will be.

        // the lookahead time is proportional to the distance between the evader
        // and the pursuer; and is inversely proportional to the sum of the
        // agent's velocities
        let LookAheadTime = ToEvader.length() / (vehicle.borrow().max_speed() + evader.speed());

        // now seek to the predicted future position of the evader
        return SteeringBehavior::Seek(vehicle, evader.position() + evader.velocity() * LookAheadTime);
    }

    //----------------------------- Evade ------------------------------------
    //
    //  similar to pursuit except the agent Flees from the estimated future
    //  position of the pursuer
    //------------------------------------------------------------------------
    pub fn Evade(vehicle: &Rc<RefCell<Vehicle>>, pursuer: Ref<Vehicle>) -> Vec2 {
        /* Not necessary to include the check for facing direction this time */

        let ToPursuer = pursuer.position() - vehicle.borrow().position();

        // uncomment the following two lines to have Evade only consider pursuers
        // within a 'threat range'
        let ThreatRange: f32 = 100.0;
        if ToPursuer.length_squared() > ThreatRange * ThreatRange {
            return Vec2::default();
        }

        // the lookahead time is proportional to the distance between the pursuer
        // and the pursuer; and is inversely proportional to the sum of the
        // agents' velocities
        let LookAheadTime = ToPursuer.length() / (vehicle.borrow().max_speed() + pursuer.speed());

        // now flee away from predicted future position of the pursuer
        return SteeringBehavior::Flee(vehicle, pursuer.position() + pursuer.velocity() * LookAheadTime);
    }

    //--------------------------- Wander -------------------------------------
    //
    //  This behavior makes the agent wander about randomly
    //------------------------------------------------------------------------
    pub fn Wander(&mut self, vehicle: &Rc<RefCell<Vehicle>>) -> Vec2 {
        // use a timer to slow down the frequency of direction changes
        self.wander_direction_time -= vehicle.borrow().m_dTimeElapsed;

        // if self.wander_direction_time < 0.0 {
        self.wander_direction_time = RandInRange(0.05, 0.3);

        // this behavior is dependent on the update rate, so this line must
        // be included when using time independent framerate.
        let jitter_this_time_slice = self.m_dWanderJitter * vehicle.borrow().m_dTimeElapsed;

        // first, add a small random vector to the target's position
        let x_rand = RandomClamped() * jitter_this_time_slice;
        let y_rand = RandomClamped() * jitter_this_time_slice;

        // use a normal distribution for turns
        // let normal: Normal<f32> = Normal::new(0.0, 0.1).unwrap();
        // let x_rand = normal.sample(&mut thread_rng()) * jitter_this_time_slice;
        // let y_rand = normal.sample(&mut thread_rng()) * jitter_this_time_slice;
        let mut rand_vec = vec2(x_rand, y_rand);

        // if vehicle.borrow().id() == 0 {
        //     rand_vec = vec2(0.0, 0.0);
        //     self.m_vWanderTarget = rand_vec;
        // } else {
        self.m_vWanderTarget += rand_vec;
        // }

        // reproject this new vector back on to a unit circle
        self.m_vWanderTarget = self.m_vWanderTarget.normalize_or_zero();

        // increase the length of the vector to the same as the radius
        // of the wander circle
        self.m_vWanderTarget *= self.m_dWanderRadius;
        // }

        // move the target into a position WanderDist in front of the agent
        let wander_target = self.m_vWanderTarget + vec2(self.m_dWanderDistance, 0.0);

        // project the target into world space
        let world_target = PointToWorldSpace(
            wander_target,
            vehicle.borrow().heading(),
            vehicle.borrow().side(),
            vehicle.borrow().position(),
        );

        //and steer towards it
        let mut steer_force = world_target - vehicle.borrow().position();

        // if vehicle.borrow().id() == 0 {
        //     // steer_force *= 70.0;
        //     println!("\nrand_vec: {:?}", rand_vec);
        //     println!("self.m_vWanderTarget: {:?}", self.m_vWanderTarget);
        //     println!("wander_target: {:?}", wander_target);
        //     println!("world_target: {:?}", world_target);
        //     println!("steer_force: {:?}", steer_force);
        // }

        steer_force
    }

    //---------------------- ObstacleAvoidance -------------------------------
    //
    //  Given a vector of CObstacles, this method returns a steering force
    //  that will prevent the agent colliding with the closest obstacle
    //------------------------------------------------------------------------
    pub fn ObstacleAvoidance(obstacles: Vec<impl EntityMovable>) -> Vec2 {
        todo!();
        /*
        //the detection box length is proportional to the agent's velocity
            m_dDBoxLength = Prm.MinDetectionBoxLength +
            (vehicle.Speed() / vehicle.MaxSpeed()) *
            Prm.MinDetectionBoxLength;

        //tag all obstacles within range of the box for processing
            vehicle->World() -> TagObstaclesWithinViewRange(vehicle, m_dDBoxLength);

        //this will keep track of the closest intersecting obstacle (CIB)
            BaseGameEntity* ClosestIntersectingObstacle = NULL;

        //this will be used to track the distance to the CIB
            float DistToClosestIP = Maxfloat;

        //this will record the transformed local coordinates of the CIB
            Vec2 LocalPosOfClosestObstacle;

            std::vector<BaseGameEntity* >::const_iterator curOb = obstacles.begin();

            while (curOb != obstacles.end())
            {
            //if the obstacle has been tagged within range proceed
            if (( * curOb) -> IsTagged())
            {
            //calculate this obstacle's position in local space
            Vec2 LocalPos = PointToLocalSpace(( * curOb).Pos(),
            vehicle -> Heading(),
            vehicle -> Side(),
            vehicle.Pos());

        //if the local position has a negative x value then it must lay
        //behind the agent. (in which case it can be ignored)
            if (LocalPos.x > = 0)
            {
            //if the distance from the x axis to the object's position is less
        //than its radius + half the width of the detection box then there
        //is a potential intersection.
            float ExpandedRadius = ( * curOb) -> BRadius() + vehicle -> BRadius();

            if (fabs(LocalPos.y) < ExpandedRadius)
            {
            //now to do a line/circle intersection test. The center of the
        //circle is represented by (cX, cY). The intersection points are
        //given by the formula x = cX +/-sqrt(r^2-cY^2) for y=0.
        //We only need to look at the smallest positive value of x because
        //that will be the closest point of intersection.
            float cX = LocalPos.x;
            float cY = LocalPos.y;

        //we only need to calculate the sqrt part of the above equation once
            float SqrtPart = sqrt(ExpandedRadius* ExpandedRadius - cY * cY);

            float ip = cX - SqrtPart;

            if (ip < = 0.0)
            {
            ip = cX + SqrtPart;
            }

            //test to see if this is the closest so far. If it is keep a
        //record of the obstacle and its local coordinates
            if (ip < DistToClosestIP)
            {
            DistToClosestIP = ip;
            ClosestIntersectingObstacle = * curOb;
            LocalPosOfClosestObstacle = LocalPos;
            }
            }
            }
            }

            + + curOb;
            }

        //if we have found an intersecting obstacle, calculate a steering
        //force away from it
            Vec2 SteeringForce;

            if (ClosestIntersectingObstacle)
            {
            //the closer the agent is to an object, the stronger the
        //steering force should be
            float multiplier = 1.0 + (m_dDBoxLength - LocalPosOfClosestObstacle.x) / m_dDBoxLength;

        //calculate the lateral force
            SteeringForce.y = (ClosestIntersectingObstacle -> BRadius() - LocalPosOfClosestObstacle.y) * multiplier;

        //apply a braking force proportional to the obstacles distance from
        //the vehicle.
            const float BrakingWeight = 0.2;

            SteeringForce.x = (ClosestIntersectingObstacle ->BRadius() - LocalPosOfClosestObstacle.x) * BrakingWeight;
            }

        //finally, convert the steering vector from local to world space
            return VectorToWorldSpace(SteeringForce,
            vehicle.Heading(),
            vehicle->Side());

                */
    }

    //--------------------------- WallAvoidance --------------------------------
    //
    //  This returns a steering force that will keep the agent away from any
    //  walls it may encounter
    //------------------------------------------------------------------------
    pub fn WallAvoidance(walls: Vec<Wall2D>) -> Vec2 {
        todo!();
        /*
        //the feelers are contained in a std::vector, m_Feelers
        CreateFeelers();

        float DistToThisIP    = 0.0;
        float DistToClosestIP = Maxfloat;

        //this will hold an index into the vector of walls
        int ClosestWall = - 1;

        Vec2 SteeringForce,
        point,         //used for storing temporary info
        ClosestPoint;  //holds the closest intersection point

        //examine each feeler in turn
        for (unsigned int flr=0; flr<m_Feelers.size(); + + flr)
        {
        //run through each wall checking for any intersection points
        for (unsigned int w = 0; w <walls.size(); + + w)
        {
        if (LineIntersection2D(vehicle.Pos(),
        m_Feelers[flr],
        walls[w].From(),
        walls[w].To(),
        DistToThisIP,
        point))
        {
        //is this the closest found so far? If so keep a record
        if (DistToThisIP < DistToClosestIP)
        {
        DistToClosestIP = DistToThisIP;
        ClosestWall = w;
        ClosestPoint = point;                }
        }
        }//next wall


        //if an intersection point has been detected, calculate a force
        //that will direct the agent away
        if (ClosestWall > = 0)
        {
        //calculate by what distance the projected position of the agent
        //will overshoot the wall
        Vec2 OverShoot = m_Feelers[flr] - ClosestPoint;

        //create a force in the direction of the wall normal, with a
        //magnitude of the overshoot
        SteeringForce = walls[ClosestWall].Normal() * OverShoot.Length();
        }

        }//next feeler

        return SteeringForce;

             */
    }

    //------------------------------- CreateFeelers --------------------------
    //
    //  Creates the antenna utilized by WallAvoidance
    //------------------------------------------------------------------------
    pub fn CreateFeelers() {
        todo!();
        /*
        //feeler pointing straight in front
        m_Feelers[0] = vehicle.Pos() + m_dWallDetectionFeelerLength * vehicle -> Heading();

        //feeler to left
        Vec2 temp = vehicle -> Heading();
        Vec2DRotateAroundOrigin(temp, HalfPi * 3.5f);
        m_Feelers[1] = vehicle.Pos() + m_dWallDetectionFeelerLength / 2.0f * temp;

        //feeler to right
        temp = vehicle -> Heading();
        Vec2DRotateAroundOrigin(temp, HalfPi * 0.5f);
        m_Feelers[2] = vehicle.Pos() + m_dWallDetectionFeelerLength /2.0f * temp;

             */
    }

    //---------------------------- Separation --------------------------------
    //
    // this calculates a force repelling from the other neighbors
    //------------------------------------------------------------------------
    pub fn Separation(neighbors: &Vec<Rc<RefCell<Vehicle>>>) -> Vec2 {
        todo!();
        /*
        Vec2 SteeringForce;

        for (unsigned int a=0; a<neighbors.size(); + + a)
        {
        //make sure this agent isn't included in the calculations and that
        //the agent being examined is close enough. ***also make sure it doesn't
        //include the evade target ***
        if ((neighbors[a] != vehicle) & & neighbors[a] -> IsTagged() & &
        (neighbors[a] != m_pTargetAgent1))
        {
        Vec2 ToAgent = vehicle.Pos() - neighbors[a].Pos();

        //scale the force inversely proportional to the agents distance
        //from its neighbor.
        SteeringForce += Vec2DNormalize(ToAgent) / ToAgent.Length();

        }
        }

        return SteeringForce;

             */
    }

    //---------------------------- Alignment ---------------------------------
    //
    //  returns a force that attempts to align this agents heading with that
    //  of its neighbors
    //------------------------------------------------------------------------
    pub fn Alignment(neighbors: &Vec<Rc<RefCell<Vehicle>>>) -> Vec2 {
        todo!();
        /*
        //used to record the average heading of the neighbors
        Vec2 AverageHeading;

        //used to count the number of vehicles in the neighborhood
        int    NeighborCount = 0;

        //iterate through all the tagged vehicles and sum their heading vectors
        for (unsigned int a=0; a<neighbors.size(); + + a)
        {
        //make sure *this* agent isn't included in the calculations and that
        //the agent being examined  is close enough ***also make sure it doesn't
        //include any evade target ***
        if ((neighbors[a] != vehicle) & & neighbors[a] -> IsTagged() & &
        (neighbors[a] != m_pTargetAgent1))
        {
        AverageHeading += neighbors[a] -> Heading();

        + + NeighborCount;
        }
        }

        //if the neighborhood contained one or more vehicles, average their
        //heading vectors.
        if (NeighborCount > 0)
        {
        AverageHeading /= (float)NeighborCount;

        AverageHeading -= vehicle-> Heading();
        }

        return AverageHeading;

             */
    }

    //-------------------------------- Cohesion ------------------------------
    //
    //  returns a steering force that attempts to move the agent towards the
    //  center of mass of the agents in its immediate area
    //------------------------------------------------------------------------
    pub fn Cohesion(
        vehicle: &Rc<RefCell<Vehicle>>,
        m_pTargetAgent1: &Option<Rc<RefCell<Vehicle>>>,
        neighbors: &Vec<Rc<RefCell<Vehicle>>>,
    ) -> Vec2 {
        // first find the center of mass of all the agents
        let mut center_of_mass: Vec2 = Default::default();
        let mut SteeringForce: Vec2 = Default::default();

        let mut NeighborCount: i32 = 0;

        //iterate through the neighbors and sum up all the position vectors
        for neighbor in neighbors {
            // make sure *this* agent isn't included in the calculations and that
            // the agent being examined is close enough ***also make sure it doesn't
            // include the evade target ***
            let is_target_agent = if let Some(agent) = m_pTargetAgent1 {
                agent.borrow().id() == neighbor.borrow().id()
            } else {
                false
            };

            if (neighbor.borrow().id() != vehicle.borrow().id()) && neighbor.borrow().is_tagged() && (!is_target_agent) {
                center_of_mass += neighbor.borrow().position();

                NeighborCount += 1;
            }
        }

        if NeighborCount > 0 {
            // the center of mass is the average of the sum of positions
            center_of_mass = center_of_mass.div(NeighborCount as f32);

            // now seek towards that position
            SteeringForce = SteeringBehavior::Seek(vehicle, center_of_mass);
        }

        // the magnitude of cohesion is usually much larger than separation or
        // alignment so it usually helps to normalize it.
        return SteeringForce.normalize_or_zero();
    }

    /* NOTE: the next three behaviors are the same as the above three, except
    that they use a cell-space partition to find the neighbors
    */

    //---------------------------- Separation --------------------------------
    //
    // this calculates a force repelling from the other neighbors
    //
    //  USES SPACIAL PARTITIONING
    //------------------------------------------------------------------------
    pub fn SeparationPlus(vehicle: &Rc<RefCell<Vehicle>>, neighbors: &Vec<Rc<RefCell<dyn EntityMovable>>>) -> Vec2 {
        let mut SteeringForce = Vec2::default();

        // iterate through the neighbors and sum up all the position vectors
        for pv in neighbors.iter() {
            if pv.borrow().id() != vehicle.borrow().id() {
                let to_agent = vehicle.borrow().position() - pv.borrow().position();
                // scale the force inversely proportional to the agents distance from its neighbor.
                SteeringForce += to_agent.normalize_or_zero() / to_agent.length();
            }
        }
        SteeringForce
    }

    //---------------------------- Alignment ---------------------------------
    //
    //  returns a force that attempts to align this agents heading with that
    //  of its neighbors
    //
    //  USES SPACIAL PARTITIONING
    //------------------------------------------------------------------------
    pub fn AlignmentPlus(vehicle: &Rc<RefCell<Vehicle>>, neighbors: &Vec<Rc<RefCell<dyn EntityMovable>>>) -> Vec2 {
        // This will record the average heading of the neighbors
        let mut AverageHeading = Vec2::default();

        // This count the number of vehicles in the neighborhood
        let mut NeighborCount: f32 = 0.0;

        for pv in neighbors.iter() {
            if pv.borrow().id() != vehicle.borrow().id() {
                AverageHeading += pv.borrow().heading();
                NeighborCount += 1.0;
            }
        }

        if NeighborCount > 0.0 {
            AverageHeading /= NeighborCount;
            AverageHeading -= vehicle.borrow().heading();
        }

        AverageHeading
    }

    //-------------------------------- Cohesion ------------------------------
    //
    //  returns a steering force that attempts to move the agent towards the
    //  center of mass of the agents in its immediate area
    //
    //  USES SPACIAL PARTITIONING
    //------------------------------------------------------------------------
    pub fn CohesionPlus(vehicle: &Rc<RefCell<Vehicle>>, neighbors: &Vec<Rc<RefCell<dyn EntityMovable>>>) -> Vec2 {
        // first find the center of mass of all the agents
        let mut CenterOfMass = Vec2::default();
        let mut SteeringForce = Vec2::default();

        let mut NeighborCount = 0;

        // iterate through the neighbors and sum up all the position vectors
        for pv in neighbors.iter() {
            //make sure *this* agent isn't included in the calculations and that
            //the agent being examined is close enough
            if pv.borrow().id() != vehicle.borrow().id() {
                CenterOfMass += pv.borrow().position();
                NeighborCount += 1;
            }
        }

        if NeighborCount > 0 {
            //the center of mass is the average of the sum of positions
            CenterOfMass /= NeighborCount as f32;
            //now seek towards that position
            SteeringForce = SteeringBehavior::Seek(vehicle, CenterOfMass);
        }

        //the magnitude of cohesion is usually much larger than separation or
        //alignment so it usually helps to normalize it.
        SteeringForce.normalize_or_zero()
    }

    //--------------------------- Interpose ----------------------------------
    //
    //  Given two agents, this method returns a force that attempts to
    //  position the vehicle between them
    //------------------------------------------------------------------------
    pub fn Interpose(AgentA: &Vehicle, AgentB: &Vehicle) -> Vec2 {
        todo!();
        /*
        //first we need to figure out where the two agents are going to be at
        //time T in the future. This is approximated by determining the time
        //taken to reach the mid way point at the current time at at max speed.
        Vec2 MidPoint = (AgentA.Pos() + AgentB.Pos()) / 2.0;

        float TimeToReachMidPoint = Vec2DDistance(vehicle.Pos(), MidPoint) /
        vehicle.MaxSpeed();

        //now we have T, we assume that agent A and agent B will continue on a
        //straight trajectory and extrapolate to get their future positions
        Vec2 APos = AgentA.Pos() + AgentA.Velocity() * TimeToReachMidPoint;
        Vec2 BPos = AgentB.Pos() + AgentB.Velocity() * TimeToReachMidPoint;

        //calculate the mid point of these predicted positions
        MidPoint = (APos + BPos) / 2.0;

        //then steer to Arrive at it
        return Arrive(MidPoint, fast);

             */
    }

    //--------------------------- Hide ---------------------------------------
    //
    //------------------------------------------------------------------------
    pub fn Hide(hunter: &Vehicle, obstacles: Vec<impl EntityMovable>) -> Vec2 {
        todo!();
        /*
        float    DistToClosest = Maxfloat;
        Vec2 BestHidingSpot;

        std::vector<BaseGameEntity* >::const_iterator curOb = obstacles.begin();
        std::vector<BaseGameEntity* >::const_iterator closest;

        while (curOb != obstacles.end())
        {
        //calculate the position of the hiding spot for this obstacle
        Vec2 HidingSpot = GetHidingPosition(( * curOb).Pos(),
        ( * curOb) -> BRadius(),
        hunter.Pos());

        //work in distance-squared space to find the closest hiding
        //spot to the agent
        float dist = Vec2DDistanceSq(HidingSpot, vehicle.Pos());

        if (dist < DistToClosest)
        {
        DistToClosest = dist;
        BestHidingSpot = HidingSpot;
        closest = curOb;
        }
        + + curOb;

        }//end while

        //if no suitable obstacles found then Evade the hunter
        if (DistToClosest == MaxFloat)
        {
        return Evade(hunter);
        }

        //else use Arrive on the hiding spot
        return Arrive(BestHidingSpot, fast);

             */
    }

    //------------------------- GetHidingPosition ----------------------------
    //
    //  Given the position of a hunter, and the position and radius of
    //  an obstacle, this method calculates a position DistanceFromBoundary
    //  away from its bounding radius and directly opposite the hunter
    //------------------------------------------------------------------------
    pub fn GetHidingPosition(posOb: Vec2, radiusOb: f32, posHunter: Vec2) -> Vec2 {
        todo!();
        /*
        //calculate how far away the agent is to be from the chosen obstacle's
        //bounding radius
        const float DistanceFromBoundary = 30.0;
        float       DistAway    = radiusOb + DistanceFromBoundary;

        //calculate the heading toward the object from the hunter
        Vec2 ToOb(posOb - posHunter);
        ToOb.normalize_or_zero();

        //scale it to size and add to the obstacles position to get
        //the hiding spot.
        return (ToOb * DistAway) + posOb;

             */
    }

    //------------------------------- FollowPath -----------------------------
    //
    //  Given a series of Vec2s, this method produces a force that will
    //  move the agent along the waypoints in order. The agent uses the
    // 'Seek' behavior to move to the next waypoint - unless it is the last
    //  waypoint, in which case it 'Arrives'
    //------------------------------------------------------------------------
    pub fn FollowPath() -> Vec2 {
        todo!();
        /*
        //move to next target if close enough to current target (working in
        //distance squared space)
        if (Vec2DDistanceSq(m_pPath->CurrentWaypoint(), vehicle.Pos()) <
            m_dWaypointSeekDistSq)
        {
            m_pPath -> SetNextWaypoint();
        }

        if (!m_pPath -> Finished())
        {
            return Seek(m_pPath->CurrentWaypoint());
        }
        else
        {
            return Arrive(m_pPath->CurrentWaypoint(), normal);
        }

         */
    }

    //------------------------- Offset Pursuit -------------------------------
    //
    //  Produces a steering force that keeps a vehicle at a specified offset
    //  from a leader vehicle
    //------------------------------------------------------------------------
    pub fn OffsetPursuit(leader: &Vehicle, offset: Vec2) -> Vec2 {
        todo!();
        /*
        //calculate the offset's position in world space
        Vec2 WorldOffsetPos = PointToWorldSpace(offset,
        leader.Heading(),
        leader->Side(),
        leader.Pos());

        Vec2 ToOffset = WorldOffsetPos - vehicle.Pos();

        //the lookahead time is propotional to the distance between the leader
        //and the pursuer; and is inversely proportional to the sum of both
        //agent's velocities
        float LookAheadTime = ToOffset.Length() / (vehicle.MaxSpeed() + leader.Speed());

        //now Arrive at the predicted future position of the offset
        return Arrive(WorldOffsetPos + leader.Velocity() * LookAheadTime, fast);

             */
    }

    // bool KeyDown(char key)
    // {
    // return true;
    // }
    //
    // #define KEYDOWN KeyDown
    // #define VK_INSERT 'I'
    // #define VK_DELETE 'D'
    // #define VK_HOME 'H'
    // #define VK_END 'E'

    //for receiving keyboard input from user
    //#define KEYDOWN(vk_code) ((GetAsyncKeyState(vk_code) & 0x8000) ? 1 : 0)
    //----------------------------- RenderAids -------------------------------
    //
    //------------------------------------------------------------------------
    pub fn RenderAids() {
        /*
        gdi->TransparentText();
        gdi->TextColor(Cgl::grey);

        int NextSlot = 0; int SlotSize = 20;

        if (KEYDOWN(VK_INSERT))
        {
            vehicle->SetMaxForce(vehicle->MaxForce() + 1000.0f*vehicle->TimeElapsed());
        }

        if (KEYDOWN(VK_DELETE))
        {
            if (vehicle->MaxForce() > 0.2f)
                vehicle->SetMaxForce(vehicle->MaxForce() - 1000.0f*vehicle->TimeElapsed());
        }

        if (KEYDOWN(VK_HOME))
        {
            vehicle->SetMaxSpeed(vehicle.MaxSpeed() + 50.0f*vehicle->TimeElapsed());
        }

        if (KEYDOWN(VK_END))
        {
            if (vehicle.MaxSpeed() > 0.2f)
                vehicle->SetMaxSpeed(vehicle.MaxSpeed() - 50.0f*vehicle->TimeElapsed());
        }

        if (vehicle->MaxForce() < 0)
            vehicle->SetMaxForce(0.0f);

        if (vehicle.MaxSpeed() < 0)
            vehicle->SetMaxSpeed(0.0f);

        if (vehicle->ID() == 0)
        {
            gdi->TextAtPos(5,NextSlot,"MaxForce(Ins/Del):");
            gdi->TextAtPos(160,NextSlot,ttos(vehicle->MaxForce()/Prm.SteeringForceTweaker));
            NextSlot+=SlotSize;
        }

        if (vehicle->ID() == 0)
        {
            gdi->TextAtPos(5,NextSlot,"MaxSpeed(Home/End):");
            gdi->TextAtPos(160,NextSlot,ttos(vehicle.MaxSpeed()));
            NextSlot+=SlotSize;
        }

        //render the steering force
        if (vehicle->World()->RenderSteeringForce())
        {
            gdi->RedPen();
            Vec2 F = (self.m_vSteeringForce / Prm.SteeringForceTweaker) * Prm.VehicleScale ;
            gdi->Line(vehicle.Pos(), vehicle.Pos() + F);
        }

        //render wander stuff if relevant
        if (On(wander) && vehicle->World()->RenderWanderCircle())
        {
            if (KEYDOWN('F')){m_dWanderJitter+=1.0f*vehicle->TimeElapsed(); Clamp(m_dWanderJitter, 0.0f, 100.0f);}
            if (KEYDOWN('V')){m_dWanderJitter-=1.0f*vehicle->TimeElapsed(); Clamp(m_dWanderJitter, 0.0f, 100.0f );}
            if (KEYDOWN('G')){m_dWanderDistance+=2.0f*vehicle->TimeElapsed(); Clamp(m_dWanderDistance, 0.0f, 50.0f);}
            if (KEYDOWN('B')){m_dWanderDistance-=2.0f*vehicle->TimeElapsed(); Clamp(m_dWanderDistance, 0.0f, 50.0f);}
            if (KEYDOWN('H')){m_dWanderRadius+=2.0f*vehicle->TimeElapsed(); Clamp(m_dWanderRadius, 0.0f, 100.0f);}
            if (KEYDOWN('N')){m_dWanderRadius-=2.0f*vehicle->TimeElapsed(); Clamp(m_dWanderRadius, 0.0f, 100.0f);}


            if (vehicle->ID() == 0){ gdi->TextAtPos(5,NextSlot, "Jitter(F/V): "); gdi->TextAtPos(160, NextSlot, ttos(m_dWanderJitter));NextSlot+=SlotSize;}
            if (vehicle->ID() == 0) {gdi->TextAtPos(5,NextSlot,"Distance(G/B): "); gdi->TextAtPos(160, NextSlot, ttos(m_dWanderDistance));NextSlot+=SlotSize;}
            if (vehicle->ID() == 0) {gdi->TextAtPos(5,NextSlot,"Radius(H/N): ");gdi->TextAtPos(160, NextSlot,  ttos(m_dWanderRadius));NextSlot+=SlotSize;}


            //calculate the center of the wander circle
            Vec2 m_vTCC = PointToWorldSpace(Vec2(m_dWanderDistance*vehicle->BRadius(), 0),
                                            vehicle.Heading(),
                                            vehicle->Side(),
                                            vehicle.Pos());
            //draw the wander circle
            gdi->GreenPen();
            gdi->HollowBrush();
            gdi->Circle(m_vTCC, m_dWanderRadius*vehicle->BRadius());

            //draw the wander target
            gdi->RedPen();
            gdi->Circle(PointToWorldSpace((m_vWanderTarget + Vec2(m_dWanderDistance,0))*vehicle->BRadius(),
                                          vehicle.Heading(),
                                          vehicle->Side(),
                                          vehicle.Pos()), 3);
        }

        //render the detection box if relevant
        if (vehicle->World()->RenderDetectionBox())
        {
            gdi->GreyPen();

            //a vertex buffer rqd for drawing the detection box
            static std::vector<Vec2> box(4);

            float length = Prm.MinDetectionBoxLength +
            (vehicle.Speed()/vehicle.MaxSpeed()) *
            Prm.MinDetectionBoxLength;

            //verts for the detection box buffer
            box[0] = Vec2(0,vehicle->BRadius());
            box[1] = Vec2(length, vehicle->BRadius());
            box[2] = Vec2(length, -vehicle->BRadius());
            box[3] = Vec2(0, -vehicle->BRadius());


            if (!vehicle->isSmoothingOn())
            {
                box = WorldTransform(box,vehicle.Pos(),vehicle.Heading(),vehicle->Side());
                gdi->ClosedShape(box);
            }
            else
            {
                box = WorldTransform(box,vehicle.Pos(),vehicle->SmoothedHeading(),vehicle->SmoothedHeading().perp());
                gdi->ClosedShape(box);
            }


            //////////////////////////////////////////////////////////////////////////
            //the detection box length is proportional to the agent's velocity
            m_dDBoxLength = Prm.MinDetectionBoxLength +
            (vehicle.Speed()/vehicle.MaxSpeed()) *
            Prm.MinDetectionBoxLength;

            //tag all obstacles within range of the box for processing
            vehicle->World()->TagObstaclesWithinViewRange(vehicle, m_dDBoxLength);

            //this will keep track of the closest intersecting obstacle (CIB)
            BaseGameEntity* ClosestIntersectingObstacle = NULL;

            //this will be used to track the distance to the CIB
            float DistToClosestIP = Maxfloat;

            //this will record the transformed local coordinates of the CIB
            Vec2 LocalPosOfClosestObstacle;

            std::vector<BaseGameEntity*>::const_iterator curOb = vehicle->World()->Obstacles().begin();

            while(curOb != vehicle->World()->Obstacles().end())
            {
                //if the obstacle has been tagged within range proceed
                if ((*curOb)->IsTagged())
                {
                    //calculate this obstacle's position in local space
                    Vec2 LocalPos = PointToLocalSpace((*curOb).Pos(),
                                                      vehicle.Heading(),
                                                      vehicle->Side(),
                                                      vehicle.Pos());

                    //if the local position has a negative x value then it must lay
                    //behind the agent. (in which case it can be ignored)
                    if (LocalPos.x >= 0)
                    {
                        //if the distance from the x axis to the object's position is less
                        //than its radius + half the width of the detection box then there
                        //is a potential intersection.
                        if (fabs(LocalPos.y) < ((*curOb)->BRadius() + vehicle->BRadius()))
                        {
                            gdi->ThickRedPen();
                            gdi->ClosedShape(box);
                        }
                    }
                }

                ++curOb;
            }


            /////////////////////////////////////////////////////
        }

        //render the wall avoidnace feelers
        if (On(wall_avoidance) && vehicle->World()->RenderFeelers())
        {
            gdi->OrangePen();

            for (unsigned int flr=0; flr<m_Feelers.size(); ++flr)
            {

                gdi->Line(vehicle.Pos(), m_Feelers[flr]);
            }
        }

        //render path info
        if (On(follow_path) && vehicle->World()->RenderPath())
        {
            m_pPath->Render();
        }


        if (On(separation))
        {
            if (vehicle->ID() == 0)
            {
                gdi->TextAtPos(5, NextSlot, "Separation(S/X):");
                gdi->TextAtPos(160, NextSlot, ttos(m_dWeightSeparation/Prm.SteeringForceTweaker));
                NextSlot+=SlotSize;
            }

            if (KEYDOWN('S'))
            {
                m_dWeightSeparation += 200*vehicle->TimeElapsed();
                Clamp(m_dWeightSeparation, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }

            if (KEYDOWN('X'))
            {
                m_dWeightSeparation -= 200*vehicle->TimeElapsed();
                Clamp(m_dWeightSeparation, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }
        }

        if (On(allignment))
        {
            if (vehicle->ID() == 0)
            {
                gdi->TextAtPos(5, NextSlot, "Alignment(A/Z):");
                gdi->TextAtPos(160, NextSlot, ttos(m_dWeightAlignment/Prm.SteeringForceTweaker));
                NextSlot+=SlotSize;
            }

            if (KEYDOWN('A'))
            {
                m_dWeightAlignment += 200*vehicle->TimeElapsed();
                Clamp(m_dWeightAlignment, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }

            if (KEYDOWN('Z'))
            {
                m_dWeightAlignment -= 200*vehicle->TimeElapsed();
                Clamp(m_dWeightAlignment, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }
        }

        if (On(cohesion))
        {
            if (vehicle->ID() == 0)
            {
                gdi->TextAtPos(5, NextSlot, "Cohesion(D/C):");
                gdi->TextAtPos(160, NextSlot, ttos(m_dWeightCohesion/Prm.SteeringForceTweaker));
                NextSlot+=SlotSize;
            }

            if (KEYDOWN('D'))
            {
                m_dWeightCohesion += 200*vehicle->TimeElapsed();
                Clamp(m_dWeightCohesion, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }

            if (KEYDOWN('C'))
            {
                m_dWeightCohesion -= 200*vehicle->TimeElapsed();
                Clamp(m_dWeightCohesion, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }
        }

        if (On(follow_path))
        {
            float sd = sqrt(m_dWaypointSeekDistSq);
            if (vehicle->ID() == 0)
            {
                gdi->TextAtPos(5, NextSlot, "SeekDistance(D/C):");
                gdi->TextAtPos(160, NextSlot,ttos(sd));
                NextSlot+=SlotSize;
            }

            if (KEYDOWN('D'))
            {
                m_dWaypointSeekDistSq += 1.0;
            }

            if (KEYDOWN('C'))
            {
                m_dWaypointSeekDistSq -= 1.0;
                Clamp(m_dWaypointSeekDistSq, 0.0f, 400.0f);
            }
        }
        */
    }
}
