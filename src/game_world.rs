use std::cell::RefCell;
use crate::base_entity::BaseGameEntity;
use crate::cell_space_partition::CellSpacePartition;
use crate::param_loader::PRM;
use crate::path::Path;
use crate::utils::*;
use crate::vehicle::Vehicle;
use crate::wall_2d::Wall2D;
use glam::{vec2, Vec2};
use rand::thread_rng;
use std::f32::consts::TAU;
use std::rc::Rc;

pub struct GameWorld {
    //a container of all the moving entities
    m_Vehicles: Vec<Rc<RefCell<Vehicle>>>,

    //any obstacles
    m_Obstacles: Vec<BaseGameEntity>,

    //container containing any walls in the environment
    m_Walls: Vec<Wall2D>,

    m_pCellSpace: CellSpacePartition<Vehicle>,

    //any path we may create for the vehicles to follow
    m_pPath: Option<Path>,

    //set true to pause the motion
    m_bPaused: bool,

    //local copy of client window dimensions
    m_cxClient: i32,
    m_cyClient: i32,

    //the position of the crosshair
    m_vCrosshair: Vec2,

    //keeps track of the average FPS
    m_dAvFrameTime: f32,

    //flags to turn aids and obstacles etc on/off
    m_bShowWalls: bool,
    m_bShowObstacles: bool,
    m_bShowPath: bool,
    m_bShowDetectionBox: bool,
    m_bShowWanderCircle: bool,
    m_bShowFeelers: bool,
    m_bShowSteeringForce: bool,
    m_bShowFPS: bool,
    m_bRenderNeighbors: bool,
    m_bViewKeys: bool,
    m_bShowCellSpaceInfo: bool,
}

impl GameWorld {
    pub fn new(cx: i32, cy: i32) -> Rc<RefCell<GameWorld>> {
        let border = 30f32;
        let path = Path::new(5, border, border, cx as f32 - border, cy as f32 - border, true);
        let cell_space = CellSpacePartition::<Vehicle>::new(cx as f32, cy as f32, 0, 0, 0);

        let game_world = GameWorld {
            m_Vehicles: vec![],
            m_Obstacles: vec![],
            m_Walls: vec![],
            m_pCellSpace: cell_space,
            m_pPath: Some(path),
            m_bPaused: false,
            m_cxClient: cx,
            m_cyClient: cy,
            m_vCrosshair: vec2(cx as f32 / 2.0, cy as f32 / 2.0), // seems there was a bug in original code here.
            m_dAvFrameTime: 0.0,
            m_bShowWalls: false,
            m_bShowObstacles: false,
            m_bShowPath: false,
            m_bShowDetectionBox: false,
            m_bShowWanderCircle: false,
            m_bShowFeelers: false,
            m_bShowSteeringForce: false,
            m_bShowFPS: true,
            m_bRenderNeighbors: false,
            m_bViewKeys: false,
            m_bShowCellSpaceInfo: false,
        };

        let mut rng = thread_rng();

        let game_world = Rc::new(RefCell::new(game_world));
        {
            let mut game_world_mut = game_world.borrow_mut();

            // setup the agents
            for _a in 0..PRM.NumAgents {
                //determine a random starting position
                let spawn_pos = vec2(
                    cx as f32 / 2.0 + RandomClamped(&mut rng) * cx as f32 / 2.0,
                    cy as f32 / 2.0 + RandomClamped(&mut rng) * cy as f32 / 2.0,
                );

                let vehicle = Vehicle::new(
                    game_world.clone(),
                    spawn_pos,
                    RandFloat(&mut rng) * TAU,
                    vec2(0.0, 0.0),
                    PRM.VehicleMass,
                    PRM.MaxSteeringForce,
                    PRM.MaxSpeed,
                    PRM.MaxTurnRatePerSecond,
                    PRM.VehicleScale,
                );

                vehicle.borrow_mut().Steering().borrow_mut().FlockingOn();

                game_world_mut.m_Vehicles.push(vehicle.clone());
                game_world_mut.m_pCellSpace.AddEntity(vehicle.clone());
            }
        }

        game_world
    }

    pub fn cxClient(&self) -> i32 {
        self.m_cxClient
    }

    pub fn cyClient(&self) -> i32 {
        self.m_cyClient
    }
}
