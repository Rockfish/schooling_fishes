//--------------------------- Constants ----------------------------------

use std::cell::RefCell;
use std::f32::consts::TAU;
use crate::path::Path;
use crate::vehicle::Vehicle;
use glam::{Vec2, vec2};
use std::rc::Rc;
use rand::thread_rng;
use crate::base_entity::EntityBase;
use crate::param_loader::PRM;
use crate::utils::RandFloat;

//the radius of the constraining circle for the wander behavior
const WANDER_RAD: f32 = 1.2;
//distance the wander circle is projected in front of the agent
const WANDER_DIST: f32 = 2.0;
//the maximum amount of displacement along the circle each frame
const WANDER_JITTER_PER_SEC: f32 = 80.0;
//used in path following
const WAYPOINT_SEEK_DIST: f32 = 20.0;

//------------------------------------------------------------------------

pub enum Deceleration {
    slow = 3,
    normal = 2,
    fast = 1,
}

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

    pub fn WanderOn(&mut self) { self.m_iFlags |= BehaviorType::wander as i32; }
    pub fn CohesionOn(&mut self) { self.m_iFlags |= BehaviorType::cohesion as i32; }
    pub fn SeparationOn(&mut self) { self.m_iFlags |= BehaviorType::separation as i32; }
    pub fn AlignmentOn(&mut self) { self.m_iFlags |= BehaviorType::alignment as i32; }

    //this function tests if a specific bit of m_iFlags is set
    pub fn On(&self, bt: BehaviorType) -> bool {
        (self.m_iFlags & bt as i32) == bt as i32
    }

    pub fn Calculate(&mut self) -> Vec2 {
        //reset the steering force
        self.m_vSteeringForce.x = 0.0;
        self.m_vSteeringForce.y = 0.0;

        if !self.m_bCellSpaceOn {

            if self.On(BehaviorType::separation) || self.On(BehaviorType::alignment) || self.On(BehaviorType::cohesion) {
                self.m_pVehicle.borrow().m_pWorld.borrow_mut().TagVehiclesWithinViewRange(&self.m_pVehicle, self.m_dViewDistance);
            }
        } else {
            //calculate neighbours in cell-space if any of the following 3 group
            //behaviors are switched on
            if self.On(BehaviorType::separation) || self.On(BehaviorType::alignment) || self.On(BehaviorType::cohesion) {
               self.m_pVehicle.borrow().m_pWorld.borrow_mut().m_pCellSpace.CalculateNeighbors(self.m_pVehicle.borrow().Pos(), self.m_dViewDistance);
            }
        }

        self.m_vSteeringForce.clone()
    }

}
