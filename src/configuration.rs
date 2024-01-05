use std::f32::consts::PI;

const STEERING_FORCE_TWEAKER: f32 = 200.0;

pub struct ConfigLoader {
    pub NumAgents: i32,
    pub NumObstacles: i32,
    pub MinObstacleRadius: f32,
    pub MaxObstacleRadius: f32,

    //number of horizontal cells used for spatial partitioning
    pub NumCellsX: i32,
    //number of vertical cells used for spatial partitioning
    pub NumCellsY: i32,

    //how many samples the smoother will use to average a value
    pub NumSamplesForSmoothing: i32,

    //used to tweak the combined steering force (simply altering the MaxSteeringForce
    //will NOT work!This tweaker affects all the steering force multipliers
    //too).
    pub SteeringForceTweaker: f32,

    pub MaxSteeringForce: f32,
    pub MaxSpeed: f32,
    pub VehicleMass: f32,

    pub Scale: f32,
    pub MaxTurnRatePerSecond: f32,

    pub SeparationWeight: f32,
    pub AlignmentWeight: f32,
    pub CohesionWeight: f32,
    pub ObstacleAvoidanceWeight: f32,
    pub WallAvoidanceWeight: f32,
    pub WanderWeight: f32,
    pub SeekWeight: f32,
    pub FleeWeight: f32,
    pub ArriveWeight: f32,
    pub PursuitWeight: f32,
    pub OffsetPursuitWeight: f32,
    pub InterposeWeight: f32,
    pub HideWeight: f32,
    pub EvadeWeight: f32,
    pub FollowPathWeight: f32,

    //how close a neighbour must be before an agent perceives it (considers it
    //to be within its neighborhood)
    pub ViewDistance: f32,

    //used in obstacle avoidance
    pub MinDetectionBoxLength: f32,

    //used in wall avoidance
    pub WallDetectionFeelerLength: f32,

    //these are the probabilities that a steering behavior will be used
    //when the prioritized dither calculate method is used
    pub prWallAvoidance: f32,
    pub prObstacleAvoidance: f32,
    pub prSeparation: f32,
    pub prAlignment: f32,
    pub prCohesion: f32,
    pub prWander: f32,
    pub prSeek: f32,
    pub prFlee: f32,
    pub prEvade: f32,
    pub prHide: f32,
    pub prArrive: f32,
}

pub const CONFIG: ConfigLoader = ConfigLoader {
    NumAgents: 250,
    NumObstacles: 7,
    MinObstacleRadius: 10.0,
    MaxObstacleRadius: 30.0,

    NumCellsX: 7,
    NumCellsY: 7,

    NumSamplesForSmoothing: 10,

    SteeringForceTweaker: STEERING_FORCE_TWEAKER,
    MaxSteeringForce: 2.0 * STEERING_FORCE_TWEAKER,
    MaxSpeed: 80.0, //150.0,

    VehicleMass: 1.0,
    Scale: 20.0, //3.5,

    SeparationWeight: 1.0 * STEERING_FORCE_TWEAKER,
    AlignmentWeight: 1.0 * STEERING_FORCE_TWEAKER,
    CohesionWeight: 2.0 * STEERING_FORCE_TWEAKER,
    ObstacleAvoidanceWeight: 10.0 * STEERING_FORCE_TWEAKER,
    WallAvoidanceWeight: 10.0 * STEERING_FORCE_TWEAKER,
    WanderWeight: 1.0 * STEERING_FORCE_TWEAKER,
    SeekWeight: 1.0 * STEERING_FORCE_TWEAKER,
    FleeWeight: 1.0 * STEERING_FORCE_TWEAKER,
    ArriveWeight: 1.0 * STEERING_FORCE_TWEAKER,
    PursuitWeight: 1.0 * STEERING_FORCE_TWEAKER,
    OffsetPursuitWeight: 1.0 * STEERING_FORCE_TWEAKER,
    InterposeWeight: 1.0 * STEERING_FORCE_TWEAKER,
    HideWeight: 1.0 * STEERING_FORCE_TWEAKER,
    EvadeWeight: 1.0 * STEERING_FORCE_TWEAKER,
    FollowPathWeight: 1.0 * STEERING_FORCE_TWEAKER,

    ViewDistance: 50.0,
    MinDetectionBoxLength: 40.0,
    WallDetectionFeelerLength: 40.0,

    prWallAvoidance: 0.5,
    prObstacleAvoidance: 0.5,
    prSeparation: 0.2,
    prAlignment: 0.3,
    prCohesion: 0.6,
    prWander: 0.8,
    prSeek: 0.8,
    prFlee: 0.6,
    prEvade: 1.0,
    prHide: 0.8,
    prArrive: 0.5,

    MaxTurnRatePerSecond: PI,
};
