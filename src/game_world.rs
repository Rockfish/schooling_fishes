use crate::base_entity::BaseGameEntity;
use crate::cell_space_partition::CellSpacePartition;
use crate::entity_functions::TagNeighbors;
use crate::param_loader::PRM;
use crate::path::Path;
use crate::utils::*;
use crate::vehicle::Vehicle;
use crate::wall_2d::Wall2D;
use glam::{vec2, Vec2};
use rand::thread_rng;
use std::cell::RefCell;
use std::f32::consts::TAU;
use std::rc::Rc;

#[derive(Debug)]
pub struct GameWorld {
    //a container of all the moving entities
    pub(crate) m_Vehicles: Vec<Rc<RefCell<Vehicle>>>,

    //any obstacles
    m_Obstacles: Vec<BaseGameEntity>,

    //container containing any walls in the environment
    m_Walls: Vec<Wall2D>,

    pub m_pCellSpace: RefCell<CellSpacePartition<Vehicle>>,
    // flag for cell space partitioning
    m_bCellSpaceOn: bool,

    //any path we may create for the vehicles to follow
    m_pPath: Option<Path>,

    //set true to pause the motion
    m_bPaused: bool,

    //local copy of client window dimensions
    m_cxClient: i32,
    m_cyClient: i32,

    //the position of the crosshair
    pub(crate) m_vCrosshair: Vec2,

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
        let cell_space = CellSpacePartition::<Vehicle>::new(cx as f32, cy as f32, PRM.NumCellsX, PRM.NumCellsY, PRM.NumAgents);

        let game_world = GameWorld {
            m_Vehicles: vec![],
            m_Obstacles: vec![],
            m_Walls: vec![],
            m_pCellSpace: cell_space.into(),
            m_bCellSpaceOn: false,
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

                // vehicle.borrow_mut().m_pSteering.as_mut().unwrap().FlockingOn();
                if let Some(steering) = &vehicle.borrow().m_pSteering {
                    steering.borrow_mut().FlockingOn();
                }

                game_world.borrow_mut().m_Vehicles.push(vehicle.clone());
                game_world.borrow().m_pCellSpace.borrow_mut().AddEntity(vehicle.clone());
            }

            game_world.borrow_mut().ToggleSpacePartition();
        }

        /* SHOAL
        #ifdef SHOAL
        m_Vehicles[Prm.NumAgents-1]->Steering()->FlockingOff();
        m_Vehicles[Prm.NumAgents-1]->SetScale(Vector2D(10, 10));
        m_Vehicles[Prm.NumAgents-1]->Steering()->WanderOn();
        m_Vehicles[Prm.NumAgents-1]->SetMaxSpeed(70);

        for (int i=0; i<Prm.NumAgents-1; ++i)
        {
            m_Vehicles[i]->Steering()->EvadeOn(m_Vehicles[Prm.NumAgents-1]);
        }
        #endif
         */

        // TODO: the way cell space partitioning is controlled doesn't make sense
        // since CellSpacePartition object is owned by the GameWorld and not by the
        // vehicles. The cpp code checks the vehicles to find out if partition is on
        // and then has the vehicles change the gameworld's CellSpacePartition object
        // instead of just having the gameworld do it itself.
        //ToggleSpacePartition();

        //create any obstacles or walls
        //CreateObstacles();
        //CreateWalls();

        game_world
    }

    pub fn cxClient(&self) -> i32 {
        self.m_cxClient
    }

    pub fn cyClient(&self) -> i32 {
        self.m_cyClient
    }

    pub fn Update(game_world: Rc<RefCell<GameWorld>>, time_elapsed: f32) {
        //  if (m_bPaused) return;

        //create a smoother to smooth the framerate
        // let SampleRate = 10;
        //static Smoother<float> FrameRateSmoother(SampleRate, 0.0);

        game_world.borrow_mut().m_dAvFrameTime = time_elapsed; // FrameRateSmoother.Update(time_elapsed);

        for vehicle in &game_world.borrow().m_Vehicles {
            //vehicle.Update(time_elapsed);
            let old_position = Vehicle::Update(vehicle, time_elapsed);

            // This bit was in vehicle, seem to make more sense to have it here.
            if game_world.borrow().m_bCellSpaceOn {
                game_world.borrow().m_pCellSpace.borrow_mut().UpdateEntity(vehicle, &old_position);
            }
        }
    }

    pub fn ToggleSpacePartition(&mut self) {
        self.m_bCellSpaceOn = !self.m_bCellSpaceOn;

        for vehicle in &self.m_Vehicles {
            if let Some(steering) = &vehicle.borrow().m_pSteering {
                steering.borrow_mut().m_bCellSpaceOn = self.m_bCellSpaceOn;
            }
        }

        if self.m_bCellSpaceOn {
            self.m_pCellSpace.borrow_mut().EmptyCells();

            for vehicle in &self.m_Vehicles {
                self.m_pCellSpace.borrow_mut().AddEntity(vehicle.clone());
            }
        } else {
            self.m_bShowCellSpaceInfo = false;
        }
    }

    pub fn TagVehiclesWithinViewRange(&mut self, pVehicle: &Rc<RefCell<Vehicle>>, range: f32) {
        TagNeighbors(pVehicle, &mut self.m_Obstacles, range);
    }

    pub fn Render(&self) {
        for wall in &self.m_Walls {
            wall.Render(true);
        }

        // for obstacle in &self.m_Obstacles {
        // gdi->Circle(m_Obstacles[ob]->Pos(), m_Obstacles[ob]->BRadius());
        // }

        let mut first = true;
        //render the agents
        for vehicle in &self.m_Vehicles {
            vehicle.borrow_mut().Render();

            //render cell partitioning stuff
            if self.m_bShowCellSpaceInfo && first {
                // gdi->HollowBrush();
                // InvertedAABBox2D box(m_Vehicles[a]->Pos() - Vector2D(Prm.ViewDistance, Prm.ViewDistance),
                // m_Vehicles[a]->Pos() + Vector2D(Prm.ViewDistance, Prm.ViewDistance));
                // box.Render();
                //
                // gdi->RedPen();
                // CellSpace()->CalculateNeighbors(m_Vehicles[a]->Pos(), Prm.ViewDistance);
                // for (BaseGameEntity* pV = CellSpace()->begin();!CellSpace()->end();pV = CellSpace()->next())
                // {
                //     gdi->Circle(pV->Pos(), pV->BRadius());
                // }
                //
                // gdi->GreenPen();
                // gdi->Circle(m_Vehicles[a]->Pos(), Prm.ViewDistance);
            }
            first = false;
        }

        if self.m_bShowPath {
            if let Some(path) = &self.m_pPath {
                path.Render();
            }
        }

        // if self.m_bShowFPS {
        //gdi->TextColor(Cgl::grey);
        //gdi->TextAtPos(5, cyClient() - 20, ttos(1.0 / m_dAvFrameTime));
        // }

        if self.m_bShowCellSpaceInfo {
            self.m_pCellSpace.borrow().RenderCells();
        }
    }
}
