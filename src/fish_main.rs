use crate::game_world::GameWorld;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct FishMain {
    game_world: Rc<RefCell<GameWorld>>,
    last_time: f32,
}

impl FishMain {
    pub fn new() -> Self {
        FishMain {
            game_world: GameWorld::new(800, 800),
            last_time: 0.0,
        }
    }

    pub fn update_with_interval(&self, time_interval: f32) {
        println!("time_interval: {}", time_interval);
        GameWorld::Update(self.game_world.clone(), time_interval);
    }

    pub fn render(&self) {
        self.game_world.borrow().Render();
    }
}
