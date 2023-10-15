//--------------------------- Constants ----------------------------------

use crate::base_entity::EntityBase;
use crate::param_loader::PRM;
use crate::path::Path;
use crate::utils::{min, RandFloat, RandomClamped};
use crate::vehicle::Vehicle;
use crate::wall_2d::Wall2D;
use glam::{vec2, Vec2};
use rand::thread_rng;
use std::cell::{Ref, RefCell};
use std::f32::consts::TAU;
use std::ops::Div;
use std::rc::Rc;
use crate::transformations::PointToWorldSpace;

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

#[derive(Debug)]
pub struct SteeringBehavior {
    //a pointer to the owner of this instance
    m_pVehicle: Rc<RefCell<Vehicle>>,

    //the steering force created by the combined effect of all
    //the selected behaviors
    m_vSteeringForce: Vec2,

    //these can be used to keep track of friends, pursuers, or prey
    m_pTargetAgent1: Option<Rc<RefCell<Vehicle>>>,
    m_pTargetAgent2: Option<Rc<RefCell<Vehicle>>>,

    //the current target
    m_vTarget: Vec2,

    //length of the 'detection box' utilized in obstacle avoidance
    m_dDBoxLength: f32,

    //a vertex buffer to contain the feelers rqd for wall avoidance
    m_Feelers: Vec<Vec2>,

    //the length of the 'feeler/s' used in wall detection
    m_dWallDetectionFeelerLength: f32,

    //the current position on the wander circle the agent is
    //attempting to steer towards
    m_vWanderTarget: Vec2,

    //explained above
    m_dWanderJitter: f32,
    m_dWanderRadius: f32,
    m_dWanderDistance: f32,

    //multipliers. These can be adjusted to effect strength of the
    //appropriate behavior. Useful to get flocking the way you require
    //for example.
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

    //how far the agent can 'see'
    m_dViewDistance: f32,

    //pointer to any current path
    m_pPath: Path,

    //the distance (squared) a vehicle has to be from a path waypoint before
    //it starts seeking to the next waypoint
    m_dWaypointSeekDistSq: f32,

    //any offset used for formations or offset pursuit
    m_vOffset: Vec2,

    //binary flags to indicate whether or not a behavior should be active
    m_iFlags: i32,

    //Arrive makes use of these to determine how quickly a vehicle
    //should decelerate to its target
    //default
    m_Deceleration: Deceleration,

    //is cell space partitioning to be used or not?
    pub(crate) m_bCellSpaceOn: bool,

    //what type of method is used to sum any active behavior
    m_SummingMethod: SummingMethod,
}

impl SteeringBehavior {
    pub fn new(vehicle: Rc<RefCell<Vehicle>>) -> Self {
        let wander_radius = WANDER_RAD;
        let mut rng = thread_rng();
        let theta = RandFloat(&mut rng) * TAU;
        let wander_target = vec2(wander_radius * theta.cos(), wander_radius * theta.sin());

        let mut path = Path::default();
        path.LoopOn();

        SteeringBehavior {
            m_pVehicle: vehicle,
            m_iFlags: 0,
            m_dDBoxLength: PRM.MinDetectionBoxLength,
            m_dWeightCohesion: PRM.CohesionWeight,
            m_dWeightAlignment: PRM.AlignmentWeight,
            m_dWeightSeparation: PRM.SeparationWeight,
            m_dWeightObstacleAvoidance: PRM.ObstacleAvoidanceWeight,
            m_dWeightWander: PRM.WanderWeight,
            m_dWeightWallAvoidance: PRM.WallAvoidanceWeight,
            m_dViewDistance: PRM.ViewDistance,
            m_dWallDetectionFeelerLength: PRM.WallDetectionFeelerLength,
            m_Feelers: vec![], // 3, ?
            m_Deceleration: Deceleration::normal,
            m_pTargetAgent1: None,
            m_pTargetAgent2: None,
            m_dWanderDistance: WANDER_DIST,
            m_dWanderJitter: WANDER_JITTER_PER_SEC,
            m_dWanderRadius: wander_radius,
            m_dWaypointSeekDistSq: WAYPOINT_SEEK_DIST * WAYPOINT_SEEK_DIST,
            m_dWeightSeek: PRM.SeekWeight,
            m_dWeightFlee: PRM.FleeWeight,
            m_dWeightArrive: PRM.ArriveWeight,
            m_dWeightPursuit: PRM.PursuitWeight,
            m_dWeightOffsetPursuit: PRM.OffsetPursuitWeight,
            m_dWeightInterpose: PRM.InterposeWeight,
            m_dWeightHide: PRM.HideWeight,
            m_dWeightEvade: PRM.EvadeWeight,
            m_dWeightFollowPath: PRM.FollowPathWeight,
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

    pub fn WanderOn(&mut self) {
        self.m_iFlags |= BehaviorType::wander as i32;
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

    pub fn isSpacePartitioningOn(&self) -> bool {
        self.m_bCellSpaceOn
    }

    //this function tests if a specific bit of m_iFlags is set
    pub fn On(&self, bt: BehaviorType) -> bool {
        (self.m_iFlags & bt as i32) == bt as i32
    }

    pub fn Calculate(&mut self, vehicle: Rc<RefCell<Vehicle>>) -> Vec2 {
        //reset the steering force
        self.m_vSteeringForce.x = 0.0;
        self.m_vSteeringForce.y = 0.0;

        if !self.m_bCellSpaceOn {
            if self.On(BehaviorType::separation) || self.On(BehaviorType::alignment) || self.On(BehaviorType::cohesion) {
                let world = vehicle.borrow().m_pWorld.clone();

                world.borrow_mut().TagVehiclesWithinViewRange(&self.m_pVehicle, self.m_dViewDistance);
            }
        } else {
            //calculate neighbours in cell-space if any of the following 3 group
            //behaviors are switched on
            if self.On(BehaviorType::separation) || self.On(BehaviorType::alignment) || self.On(BehaviorType::cohesion) {
                let pos = self.m_pVehicle.borrow().Pos();

                vehicle
                    .borrow()
                    .m_pWorld
                    .borrow()
                    .m_pCellSpace
                    .borrow_mut()
                    .CalculateNeighbors(pos, self.m_dViewDistance);
            }
        }

        let new_steering_force = match self.m_SummingMethod {
            SummingMethod::weighted_average => self.CalculateWeightedSum(),
            SummingMethod::prioritized => self.CalculatePrioritized(vehicle),
            SummingMethod::dithered => self.CalculateDithered(),
            _ => vec2(0.0, 0.0),
        };

        if new_steering_force.x.is_nan() {
            println!("Steering force is nan");
        }

        self.m_vSteeringForce = new_steering_force;

        self.m_vSteeringForce
    }

    //--------------------- AccumulateForce ----------------------------------
    //
    //  This function calculates how much of its max steering force the
    //  vehicle has left to apply and then applies that amount of the
    //  force to add.
    //------------------------------------------------------------------------
    pub fn AccumulateForce(m_pVehicle: Rc<RefCell<Vehicle>>, mut RunningTot: &mut Vec2, ForceToAdd: Vec2) -> bool {
        //calculate how much steering force the vehicle has used so far
        let MagnitudeSoFar = RunningTot.length();

        //calculate how much steering force remains to be used by this vehicle
        let MagnitudeRemaining = m_pVehicle.borrow().moving_entity.MaxForce() - MagnitudeSoFar;

        //return false if there is no more force left to use
        if MagnitudeRemaining <= 0.0 {
            return false;
        }

        //calculate the magnitude of the force we want to add
        let MagnitudeToAdd = ForceToAdd.length();

        //if the magnitude of the sum of ForceToAdd and the running total
        //does not exceed the maximum force available to this vehicle, just
        //add together. Otherwise add as much of the ForceToAdd vector is
        //possible without going over the max.
        if MagnitudeToAdd < MagnitudeRemaining {
            *RunningTot += ForceToAdd;
        } else {
            //add it to the steering force
            *RunningTot += ForceToAdd.normalize_or_zero() * MagnitudeRemaining;
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
    pub fn CalculatePrioritized(&mut self, m_pVehicle: Rc<RefCell<Vehicle>>) -> Vec2 {
        let mut force: Vec2 = Vec2::default();
        /*
            if (On(wall_avoidance))
            {
                force = WallAvoidance(m_pVehicle->World()->Walls()) *
                m_dWeightWallAvoidance;

                if (!SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force)) return self.m_vSteeringForce;
            }
            }

            if (On(obstacle_avoidance))
            {
                force = ObstacleAvoidance(m_pVehicle->World()->Obstacles()) *
                m_dWeightObstacleAvoidance;

                if (!SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force)) return self.m_vSteeringForce;
            }
            }
        */
        if self.On(BehaviorType::evade) {
            assert!(&self.m_pTargetAgent1.is_some(), "Evade target not assigned");

            force = SteeringBehavior::Evade(m_pVehicle.clone(), self.m_pTargetAgent1.as_mut().unwrap().borrow()) * self.m_dWeightEvade;

            if !SteeringBehavior::AccumulateForce(m_pVehicle.clone(), &mut self.m_vSteeringForce, force) {
                return self.m_vSteeringForce;
            }
        }

        if self.On(BehaviorType::flee) {
            force = SteeringBehavior::Flee(m_pVehicle.clone(), m_pVehicle.borrow().m_pWorld.borrow().m_vCrosshair) * self.m_dWeightFlee;

            if !SteeringBehavior::AccumulateForce(m_pVehicle.clone(), &mut self.m_vSteeringForce, force) {
                return self.m_vSteeringForce;
            }
        }

        //these next three can be combined for flocking behavior (wander is
        //also a good behavior to add into this mix)
        if !self.isSpacePartitioningOn() {
            // if self.On(BehaviorType::separation)
            // {
            //     force = Separation(m_pVehicle->World()->Agents()) * m_dWeightSeparation;
            //
            //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
            //         return self.m_vSteeringForce;
            //     }
            // }

            // if self.On(BehaviorType::alignment)
            // {
            //     force = Alignment(m_pVehicle->World()->Agents()) * m_dWeightAlignment;
            //
            //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
            //         return self.m_vSteeringForce;
            //     }
            // }

            if self.On(BehaviorType::cohesion) {
                force = self.Cohesion(
                    m_pVehicle.clone(),
                    &self.m_pTargetAgent1,
                    &m_pVehicle.borrow().m_pWorld.borrow().m_Vehicles,
                ) * self.m_dWeightCohesion;

                if !SteeringBehavior::AccumulateForce(m_pVehicle.clone(), &mut self.m_vSteeringForce, force) {
                    return self.m_vSteeringForce;
                }
            }
        } else {
            if self.On(BehaviorType::separation)
            {
                force = SteeringBehavior::SeparationPlus(m_pVehicle.clone(), &m_pVehicle.borrow().m_pWorld.borrow().m_Vehicles) * self.m_dWeightSeparation;

                if !SteeringBehavior::AccumulateForce(m_pVehicle.clone(), &mut self.m_vSteeringForce, force) {
                    return self.m_vSteeringForce;
                }
            }

            if self.On(BehaviorType::alignment) {
                force =
                    SteeringBehavior::AlignmentPlus(m_pVehicle.clone(), &m_pVehicle.borrow().m_pWorld.borrow().m_Vehicles) * self.m_dWeightAlignment;

                if !SteeringBehavior::AccumulateForce(m_pVehicle.clone(), &mut self.m_vSteeringForce, force) {
                    return self.m_vSteeringForce;
                }
            }

            if self.On(BehaviorType::cohesion) {
                force =
                    SteeringBehavior::CohesionPlus(m_pVehicle.clone(), &m_pVehicle.borrow().m_pWorld.borrow().m_Vehicles) * self.m_dWeightCohesion;

                if !SteeringBehavior::AccumulateForce(m_pVehicle.clone(), &mut self.m_vSteeringForce, force) {
                    return self.m_vSteeringForce;
                }
            }
        }

        // if self.On(BehaviorType::seek)
        // {
        //     force = SteeringBehavior::Seek(m_pVehicle.World().Crosshair()) * m_dWeightSeek;
        //
        //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
        //         return self.m_vSteeringForce;
        //     }
        // }
        //
        //
        // if self.On(BehaviorType::arrive)
        // {
        //     force = self.Arrive(m_pVehicle.World().Crosshair(), m_Deceleration) * m_dWeightArrive;
        //
        //     if !SteeringBehavior::AccumulateForce(self.m_vSteeringForce, force) {
        //         return self.m_vSteeringForce;
        //     }
        // }

        if self.On(BehaviorType::wander)
        {
            force = self.Wander(m_pVehicle.clone()) * self.m_dWeightWander;

            if !SteeringBehavior::AccumulateForce(m_pVehicle.clone(), &mut self.m_vSteeringForce, force) {
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
        //     force = self.Hide(m_pTargetAgent1, m_pVehicle.World().Obstacles()) * self.m_dWeightHide;
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
    pub fn Seek(m_pVehicle: Rc<RefCell<Vehicle>>, TargetPos: Vec2) -> Vec2 {
        let mut desired_velocity = TargetPos - m_pVehicle.borrow().Pos();
        desired_velocity = desired_velocity.normalize_or_zero();
        desired_velocity *= m_pVehicle.borrow().moving_entity.MaxSpeed();

        desired_velocity - m_pVehicle.borrow().moving_entity.Velocity()
    }

    //----------------------------- Flee -------------------------------------
    //
    //  Does the opposite of Seek
    //------------------------------------------------------------------------
    pub fn Flee(m_pVehicle: Rc<RefCell<Vehicle>>, TargetPos: Vec2) -> Vec2 {
        //only flee if the target is within 'panic distance'. Work in distance
        //squared space.
        /* const float PanicDistanceSq = 100.0f * 100.0;
        if (Vec2DDistanceSq(m_pVehicle.Pos(), target) > PanicDistanceSq)
        {
        return Vec2(0,0);
        }
        */

        let mut desired_velocity = m_pVehicle.borrow().Pos() - TargetPos;
        desired_velocity = desired_velocity.normalize_or_zero();
        desired_velocity *= m_pVehicle.borrow().moving_entity.MaxSpeed();

        return desired_velocity - m_pVehicle.borrow().moving_entity.Velocity();
    }

    //--------------------------- Arrive -------------------------------------
    //
    //  This behavior is similar to seek but it attempts to arrive at the
    //  target with a zero velocity
    //------------------------------------------------------------------------
    pub fn Arrive(m_pVehicle: Vehicle, TargetPos: Vec2, deceleration: Deceleration) -> Vec2 {
        let ToTarget = TargetPos - m_pVehicle.Pos();

        //calculate the distance to the target
        let dist = ToTarget.length();

        if dist > 0.0 {
            //because Deceleration is enumerated as an int, this value is required
            //to provide fine tweaking of the deceleration..
            let DecelerationTweaker: f32 = 0.3;

            //calculate the speed required to reach the target given the desired
            //deceleration
            let mut speed: f32 = dist / ((deceleration as i32) as f32 * DecelerationTweaker);

            //make sure the velocity does not exceed the max
            speed = min(speed, m_pVehicle.moving_entity.MaxSpeed());

            //from here proceed just like Seek except we don't need to normalize
            //the ToTarget vector because we have already gone to the trouble
            //of calculating its length: dist.
            let desired_velocity = ToTarget * speed / dist;

            return desired_velocity - m_pVehicle.moving_entity.Velocity();
        }

        vec2(0.0, 0.0)
    }

    //------------------------------ Pursuit ---------------------------------
    //
    //  this behavior creates a force that steers the agent towards the
    //  evader
    //------------------------------------------------------------------------
    pub fn Pursuit(&self, m_pVehicle: Rc<RefCell<Vehicle>>, evader: Vehicle) -> Vec2 {
        //if the evader is ahead and facing the agent then we can just seek
        //for the evader's current position.
        let ToEvader = evader.Pos() - m_pVehicle.borrow().Pos();

        let RelativeHeading = m_pVehicle.borrow().moving_entity.Heading().dot(evader.moving_entity.Heading());

        if (ToEvader.dot(m_pVehicle.borrow().moving_entity.Heading()) > 0.0) & &(RelativeHeading < -0.95)
        //acos(0.95)=18 degs
        {
            return SteeringBehavior::Seek(m_pVehicle.clone(), evader.Pos());
        }

        //Not considered ahead so we predict where the evader will be.

        //the lookahead time is proportional to the distance between the evader
        //and the pursuer; and is inversely proportional to the sum of the
        //agent's velocities
        let LookAheadTime = ToEvader.length() / (m_pVehicle.borrow().moving_entity.MaxSpeed() + evader.moving_entity.Speed());

        //now seek to the predicted future position of the evader
        return SteeringBehavior::Seek(m_pVehicle.clone(), evader.Pos() + evader.moving_entity.Velocity() * LookAheadTime);
    }

    //----------------------------- Evade ------------------------------------
    //
    //  similar to pursuit except the agent Flees from the estimated future
    //  position of the pursuer
    //------------------------------------------------------------------------
    pub fn Evade(vehicle: Rc<RefCell<Vehicle>>, pursuer: Ref<Vehicle>) -> Vec2 {
        /* Not necessary to include the check for facing direction this time */

        let ToPursuer = pursuer.Pos() - vehicle.borrow().Pos();

        //uncomment the following two lines to have Evade only consider pursuers
        //within a 'threat range'
        let ThreatRange: f32 = 100.0;
        if ToPursuer.length_squared() > ThreatRange * ThreatRange {
            return Vec2::default();
        }

        //the lookahead time is proportional to the distance between the pursuer
        //and the pursuer; and is inversely proportional to the sum of the
        //agents' velocities
        let LookAheadTime = ToPursuer.length() / (vehicle.borrow().moving_entity.MaxSpeed() + pursuer.moving_entity.Speed());

        //now flee away from predicted future position of the pursuer
        return SteeringBehavior::Flee(vehicle, pursuer.Pos() + pursuer.moving_entity.Velocity() * LookAheadTime);
    }

    //--------------------------- Wander -------------------------------------
    //
    //  This behavior makes the agent wander about randomly
    //------------------------------------------------------------------------
    pub fn Wander(&mut self, m_pVehicle: Rc<RefCell<Vehicle>>) -> Vec2 {
        //this behavior is dependent on the update rate, so this line must
        //be included when using time independent framerate.
        let JitterThisTimeSlice = self.m_dWanderJitter * m_pVehicle.borrow().m_dTimeElapsed;

        //first, add a small random vector to the target's position
        let mut rng = thread_rng();
        self.m_vWanderTarget += vec2(RandomClamped(&mut rng) * JitterThisTimeSlice,
                                RandomClamped(&mut rng) * JitterThisTimeSlice);

        //reproject this new vector back on to a unit circle
        self.m_vWanderTarget = self.m_vWanderTarget.normalize_or_zero();

        //increase the length of the vector to the same as the radius
        //of the wander circle
        self.m_vWanderTarget *= self.m_dWanderRadius;

        //move the target into a position WanderDist in front of the agent
        let target =self. m_vWanderTarget + vec2(self.m_dWanderDistance, 0.0);

        //project the target into world space
        let Target = PointToWorldSpace(target,
                                   m_pVehicle.borrow().moving_entity.m_vHeading,
                                   m_pVehicle.borrow().moving_entity.m_vSide,
                                   m_pVehicle.borrow().Pos());

        //and steer towards it
        return Target - m_pVehicle.borrow().Pos();
    }

    //---------------------- ObstacleAvoidance -------------------------------
    //
    //  Given a vector of CObstacles, this method returns a steering force
    //  that will prevent the agent colliding with the closest obstacle
    //------------------------------------------------------------------------
    pub fn ObstacleAvoidance(obstacles: Vec<impl EntityBase>) -> Vec2 {
        todo!();
        /*
        //the detection box length is proportional to the agent's velocity
            m_dDBoxLength = Prm.MinDetectionBoxLength +
            (m_pVehicle.Speed() / m_pVehicle.MaxSpeed()) *
            Prm.MinDetectionBoxLength;

        //tag all obstacles within range of the box for processing
            m_pVehicle->World() -> TagObstaclesWithinViewRange(m_pVehicle, m_dDBoxLength);

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
            m_pVehicle -> Heading(),
            m_pVehicle -> Side(),
            m_pVehicle.Pos());

        //if the local position has a negative x value then it must lay
        //behind the agent. (in which case it can be ignored)
            if (LocalPos.x > = 0)
            {
            //if the distance from the x axis to the object's position is less
        //than its radius + half the width of the detection box then there
        //is a potential intersection.
            float ExpandedRadius = ( * curOb) -> BRadius() + m_pVehicle -> BRadius();

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
            m_pVehicle.Heading(),
            m_pVehicle->Side());

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
        if (LineIntersection2D(m_pVehicle.Pos(),
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
        m_Feelers[0] = m_pVehicle.Pos() + m_dWallDetectionFeelerLength * m_pVehicle -> Heading();

        //feeler to left
        Vec2 temp = m_pVehicle -> Heading();
        Vec2DRotateAroundOrigin(temp, HalfPi * 3.5f);
        m_Feelers[1] = m_pVehicle.Pos() + m_dWallDetectionFeelerLength / 2.0f * temp;

        //feeler to right
        temp = m_pVehicle -> Heading();
        Vec2DRotateAroundOrigin(temp, HalfPi * 0.5f);
        m_Feelers[2] = m_pVehicle.Pos() + m_dWallDetectionFeelerLength /2.0f * temp;

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
        if ((neighbors[a] != m_pVehicle) & & neighbors[a] -> IsTagged() & &
        (neighbors[a] != m_pTargetAgent1))
        {
        Vec2 ToAgent = m_pVehicle.Pos() - neighbors[a].Pos();

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
        if ((neighbors[a] != m_pVehicle) & & neighbors[a] -> IsTagged() & &
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

        AverageHeading -= m_pVehicle-> Heading();
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
        &self,
        m_pVehicle: Rc<RefCell<Vehicle>>,
        m_pTargetAgent1: &Option<Rc<RefCell<Vehicle>>>,
        neighbors: &Vec<Rc<RefCell<Vehicle>>>,
    ) -> Vec2 {
        //first find the center of mass of all the agents
        let mut center_of_mass: Vec2 = Default::default();
        let mut SteeringForce: Vec2 = Default::default();

        let mut NeighborCount: i32 = 0;

        //iterate through the neighbors and sum up all the position vectors
        for neighbor in neighbors {
            //make sure *this* agent isn't included in the calculations and that
            //the agent being examined is close enough ***also make sure it doesn't
            //include the evade target ***
            let is_target_agent = if let Some(agent) = m_pTargetAgent1 {
                agent.borrow().ID() == neighbor.borrow().ID()
            } else {
                false
            };

            if (neighbor.borrow().ID() != m_pVehicle.borrow().ID()) && neighbor.borrow().IsTagged() && (!is_target_agent) {
                center_of_mass += neighbor.borrow().Pos();

                NeighborCount += 1;
            }
        }

        if NeighborCount > 0 {
            //the center of mass is the average of the sum of positions
            center_of_mass = center_of_mass.div(NeighborCount as f32);

            //now seek towards that position
            SteeringForce = SteeringBehavior::Seek(m_pVehicle.clone(), center_of_mass);
        }

        //the magnitude of cohesion is usually much larger than separation or
        //alignment so it usually helps to normalize it.
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
    pub fn SeparationPlus(vehicle: Rc<RefCell<Vehicle>>, neighbors: &Vec<Rc<RefCell<Vehicle>>>) -> Vec2 {
        let mut SteeringForce= Vec2::default();

        //iterate through the neighbors and sum up all the position vectors
        for pv in vehicle.borrow().m_pWorld.borrow().m_pCellSpace.borrow_mut().m_Neighbors.iter() {
            if pv.borrow().ID() != vehicle.borrow().ID() {
                let to_agent = vehicle.borrow().Pos() - pv.borrow().Pos();
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
    pub fn AlignmentPlus(vehicle: Rc<RefCell<Vehicle>>, neighbors: &Vec<Rc<RefCell<Vehicle>>>) -> Vec2 {
        //This will record the average heading of the neighbors
        let mut AverageHeading = Vec2::default();

        //This count the number of vehicles in the neighborhood
        let mut NeighborCount: f32 = 0.0;

        for pv in vehicle.borrow().m_pWorld.borrow().m_pCellSpace.borrow_mut().m_Neighbors.iter() {
            if pv.borrow().ID() != vehicle.borrow().ID() {
                AverageHeading += pv.borrow().moving_entity.m_vHeading;
                NeighborCount += 1.0;
            }
        }

        if NeighborCount > 0.0 {
            AverageHeading /= NeighborCount;
            AverageHeading -= vehicle.borrow().moving_entity.m_vHeading;
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
    pub fn CohesionPlus(vehicle: Rc<RefCell<Vehicle>>, neighbors: &Vec<Rc<RefCell<Vehicle>>>) -> Vec2 {
        //first find the center of mass of all the agents
        let mut CenterOfMass = Vec2::default();
        let mut SteeringForce = Vec2::default();

        let mut NeighborCount = 0;

        //iterate through the neighbors and sum up all the position vectors
        for pv in vehicle.borrow().m_pWorld.borrow().m_pCellSpace.borrow_mut().m_Neighbors.iter() {
            //make sure *this* agent isn't included in the calculations and that
            //the agent being examined is close enough
            if pv.borrow().ID() != vehicle.borrow().ID() {
                CenterOfMass += pv.borrow().Pos();
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

        float TimeToReachMidPoint = Vec2DDistance(m_pVehicle.Pos(), MidPoint) /
        m_pVehicle.MaxSpeed();

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
    pub fn Hide(hunter: &Vehicle, obstacles: Vec<impl EntityBase>) -> Vec2 {
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
        float dist = Vec2DDistanceSq(HidingSpot, m_pVehicle.Pos());

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
        if (Vec2DDistanceSq(m_pPath->CurrentWaypoint(), m_pVehicle.Pos()) <
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

        Vec2 ToOffset = WorldOffsetPos - m_pVehicle.Pos();

        //the lookahead time is propotional to the distance between the leader
        //and the pursuer; and is inversely proportional to the sum of both
        //agent's velocities
        float LookAheadTime = ToOffset.Length() / (m_pVehicle.MaxSpeed() + leader.Speed());

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
            m_pVehicle->SetMaxForce(m_pVehicle->MaxForce() + 1000.0f*m_pVehicle->TimeElapsed());
        }

        if (KEYDOWN(VK_DELETE))
        {
            if (m_pVehicle->MaxForce() > 0.2f)
                m_pVehicle->SetMaxForce(m_pVehicle->MaxForce() - 1000.0f*m_pVehicle->TimeElapsed());
        }

        if (KEYDOWN(VK_HOME))
        {
            m_pVehicle->SetMaxSpeed(m_pVehicle.MaxSpeed() + 50.0f*m_pVehicle->TimeElapsed());
        }

        if (KEYDOWN(VK_END))
        {
            if (m_pVehicle.MaxSpeed() > 0.2f)
                m_pVehicle->SetMaxSpeed(m_pVehicle.MaxSpeed() - 50.0f*m_pVehicle->TimeElapsed());
        }

        if (m_pVehicle->MaxForce() < 0)
            m_pVehicle->SetMaxForce(0.0f);

        if (m_pVehicle.MaxSpeed() < 0)
            m_pVehicle->SetMaxSpeed(0.0f);

        if (m_pVehicle->ID() == 0)
        {
            gdi->TextAtPos(5,NextSlot,"MaxForce(Ins/Del):");
            gdi->TextAtPos(160,NextSlot,ttos(m_pVehicle->MaxForce()/Prm.SteeringForceTweaker));
            NextSlot+=SlotSize;
        }

        if (m_pVehicle->ID() == 0)
        {
            gdi->TextAtPos(5,NextSlot,"MaxSpeed(Home/End):");
            gdi->TextAtPos(160,NextSlot,ttos(m_pVehicle.MaxSpeed()));
            NextSlot+=SlotSize;
        }

        //render the steering force
        if (m_pVehicle->World()->RenderSteeringForce())
        {
            gdi->RedPen();
            Vec2 F = (self.m_vSteeringForce / Prm.SteeringForceTweaker) * Prm.VehicleScale ;
            gdi->Line(m_pVehicle.Pos(), m_pVehicle.Pos() + F);
        }

        //render wander stuff if relevant
        if (On(wander) && m_pVehicle->World()->RenderWanderCircle())
        {
            if (KEYDOWN('F')){m_dWanderJitter+=1.0f*m_pVehicle->TimeElapsed(); Clamp(m_dWanderJitter, 0.0f, 100.0f);}
            if (KEYDOWN('V')){m_dWanderJitter-=1.0f*m_pVehicle->TimeElapsed(); Clamp(m_dWanderJitter, 0.0f, 100.0f );}
            if (KEYDOWN('G')){m_dWanderDistance+=2.0f*m_pVehicle->TimeElapsed(); Clamp(m_dWanderDistance, 0.0f, 50.0f);}
            if (KEYDOWN('B')){m_dWanderDistance-=2.0f*m_pVehicle->TimeElapsed(); Clamp(m_dWanderDistance, 0.0f, 50.0f);}
            if (KEYDOWN('H')){m_dWanderRadius+=2.0f*m_pVehicle->TimeElapsed(); Clamp(m_dWanderRadius, 0.0f, 100.0f);}
            if (KEYDOWN('N')){m_dWanderRadius-=2.0f*m_pVehicle->TimeElapsed(); Clamp(m_dWanderRadius, 0.0f, 100.0f);}


            if (m_pVehicle->ID() == 0){ gdi->TextAtPos(5,NextSlot, "Jitter(F/V): "); gdi->TextAtPos(160, NextSlot, ttos(m_dWanderJitter));NextSlot+=SlotSize;}
            if (m_pVehicle->ID() == 0) {gdi->TextAtPos(5,NextSlot,"Distance(G/B): "); gdi->TextAtPos(160, NextSlot, ttos(m_dWanderDistance));NextSlot+=SlotSize;}
            if (m_pVehicle->ID() == 0) {gdi->TextAtPos(5,NextSlot,"Radius(H/N): ");gdi->TextAtPos(160, NextSlot,  ttos(m_dWanderRadius));NextSlot+=SlotSize;}


            //calculate the center of the wander circle
            Vec2 m_vTCC = PointToWorldSpace(Vec2(m_dWanderDistance*m_pVehicle->BRadius(), 0),
                                            m_pVehicle.Heading(),
                                            m_pVehicle->Side(),
                                            m_pVehicle.Pos());
            //draw the wander circle
            gdi->GreenPen();
            gdi->HollowBrush();
            gdi->Circle(m_vTCC, m_dWanderRadius*m_pVehicle->BRadius());

            //draw the wander target
            gdi->RedPen();
            gdi->Circle(PointToWorldSpace((m_vWanderTarget + Vec2(m_dWanderDistance,0))*m_pVehicle->BRadius(),
                                          m_pVehicle.Heading(),
                                          m_pVehicle->Side(),
                                          m_pVehicle.Pos()), 3);
        }

        //render the detection box if relevant
        if (m_pVehicle->World()->RenderDetectionBox())
        {
            gdi->GreyPen();

            //a vertex buffer rqd for drawing the detection box
            static std::vector<Vec2> box(4);

            float length = Prm.MinDetectionBoxLength +
            (m_pVehicle.Speed()/m_pVehicle.MaxSpeed()) *
            Prm.MinDetectionBoxLength;

            //verts for the detection box buffer
            box[0] = Vec2(0,m_pVehicle->BRadius());
            box[1] = Vec2(length, m_pVehicle->BRadius());
            box[2] = Vec2(length, -m_pVehicle->BRadius());
            box[3] = Vec2(0, -m_pVehicle->BRadius());


            if (!m_pVehicle->isSmoothingOn())
            {
                box = WorldTransform(box,m_pVehicle.Pos(),m_pVehicle.Heading(),m_pVehicle->Side());
                gdi->ClosedShape(box);
            }
            else
            {
                box = WorldTransform(box,m_pVehicle.Pos(),m_pVehicle->SmoothedHeading(),m_pVehicle->SmoothedHeading().perp());
                gdi->ClosedShape(box);
            }


            //////////////////////////////////////////////////////////////////////////
            //the detection box length is proportional to the agent's velocity
            m_dDBoxLength = Prm.MinDetectionBoxLength +
            (m_pVehicle.Speed()/m_pVehicle.MaxSpeed()) *
            Prm.MinDetectionBoxLength;

            //tag all obstacles within range of the box for processing
            m_pVehicle->World()->TagObstaclesWithinViewRange(m_pVehicle, m_dDBoxLength);

            //this will keep track of the closest intersecting obstacle (CIB)
            BaseGameEntity* ClosestIntersectingObstacle = NULL;

            //this will be used to track the distance to the CIB
            float DistToClosestIP = Maxfloat;

            //this will record the transformed local coordinates of the CIB
            Vec2 LocalPosOfClosestObstacle;

            std::vector<BaseGameEntity*>::const_iterator curOb = m_pVehicle->World()->Obstacles().begin();

            while(curOb != m_pVehicle->World()->Obstacles().end())
            {
                //if the obstacle has been tagged within range proceed
                if ((*curOb)->IsTagged())
                {
                    //calculate this obstacle's position in local space
                    Vec2 LocalPos = PointToLocalSpace((*curOb).Pos(),
                                                      m_pVehicle.Heading(),
                                                      m_pVehicle->Side(),
                                                      m_pVehicle.Pos());

                    //if the local position has a negative x value then it must lay
                    //behind the agent. (in which case it can be ignored)
                    if (LocalPos.x >= 0)
                    {
                        //if the distance from the x axis to the object's position is less
                        //than its radius + half the width of the detection box then there
                        //is a potential intersection.
                        if (fabs(LocalPos.y) < ((*curOb)->BRadius() + m_pVehicle->BRadius()))
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
        if (On(wall_avoidance) && m_pVehicle->World()->RenderFeelers())
        {
            gdi->OrangePen();

            for (unsigned int flr=0; flr<m_Feelers.size(); ++flr)
            {

                gdi->Line(m_pVehicle.Pos(), m_Feelers[flr]);
            }
        }

        //render path info
        if (On(follow_path) && m_pVehicle->World()->RenderPath())
        {
            m_pPath->Render();
        }


        if (On(separation))
        {
            if (m_pVehicle->ID() == 0)
            {
                gdi->TextAtPos(5, NextSlot, "Separation(S/X):");
                gdi->TextAtPos(160, NextSlot, ttos(m_dWeightSeparation/Prm.SteeringForceTweaker));
                NextSlot+=SlotSize;
            }

            if (KEYDOWN('S'))
            {
                m_dWeightSeparation += 200*m_pVehicle->TimeElapsed();
                Clamp(m_dWeightSeparation, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }

            if (KEYDOWN('X'))
            {
                m_dWeightSeparation -= 200*m_pVehicle->TimeElapsed();
                Clamp(m_dWeightSeparation, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }
        }

        if (On(allignment))
        {
            if (m_pVehicle->ID() == 0)
            {
                gdi->TextAtPos(5, NextSlot, "Alignment(A/Z):");
                gdi->TextAtPos(160, NextSlot, ttos(m_dWeightAlignment/Prm.SteeringForceTweaker));
                NextSlot+=SlotSize;
            }

            if (KEYDOWN('A'))
            {
                m_dWeightAlignment += 200*m_pVehicle->TimeElapsed();
                Clamp(m_dWeightAlignment, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }

            if (KEYDOWN('Z'))
            {
                m_dWeightAlignment -= 200*m_pVehicle->TimeElapsed();
                Clamp(m_dWeightAlignment, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }
        }

        if (On(cohesion))
        {
            if (m_pVehicle->ID() == 0)
            {
                gdi->TextAtPos(5, NextSlot, "Cohesion(D/C):");
                gdi->TextAtPos(160, NextSlot, ttos(m_dWeightCohesion/Prm.SteeringForceTweaker));
                NextSlot+=SlotSize;
            }

            if (KEYDOWN('D'))
            {
                m_dWeightCohesion += 200*m_pVehicle->TimeElapsed();
                Clamp(m_dWeightCohesion, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }

            if (KEYDOWN('C'))
            {
                m_dWeightCohesion -= 200*m_pVehicle->TimeElapsed();
                Clamp(m_dWeightCohesion, 0.0f, 50.0f * Prm.SteeringForceTweaker);
            }
        }

        if (On(follow_path))
        {
            float sd = sqrt(m_dWaypointSeekDistSq);
            if (m_pVehicle->ID() == 0)
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
