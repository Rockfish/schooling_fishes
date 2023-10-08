//--------------------------- Constants ----------------------------------

use crate::path::Path;
use crate::vehicle::Vehicle;
use glam::Vec2;
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

enum Deceleration {
    slow = 3,
    normal = 2,
    fast = 1,
}

enum SummingMethod {
    weighted_average,
    prioritized,
    dithered,
}

enum BehaviorType {
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
    m_pVehicle: Rc<Vehicle>,

    //the steering force created by the combined effect of all
    //the selected behaviors
    m_vSteeringForce: Vec2,

    //these can be used to keep track of friends, pursuers, or prey
    m_pTargetAgent1: Rc<Vehicle>,
    m_pTargetAgent2: Rc<Vehicle>,

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
    m_pPath: Rc<Path>,

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
    m_bCellSpaceOn: bool,

    //what type of method is used to sum any active behavior
    m_SummingMethod: SummingMethod,
}
