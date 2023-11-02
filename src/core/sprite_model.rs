use crate::core::mesh::Mesh;
use crate::core::shader::Shader;
use glam::{vec2, Vec3};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum SpriteAnimationType {
    None,
    Once,
    Cycle,
    BackAndForth,
}

#[derive(Debug, Clone)]
pub struct SpriteModel {
    pub name: Rc<str>,
    pub animation_type: SpriteAnimationType,
    pub shader: Rc<Shader>,
    pub mesh: Rc<Mesh>,
    pub x_offset: u32,
    pub y_offset: u32,
    pub x_step: u32,
    pub y_step: u32,
    pub num_steps: u32,
    pub step_timer: f32,
    pub step_count: f32,
    pub step_increment: f32,
}

impl SpriteModel {
    pub fn copy(&self) -> SpriteModel {
        SpriteModel {
            name: self.name.clone(),
            animation_type: self.animation_type.clone(),
            shader: self.shader.clone(),
            mesh: self.mesh.clone(),
            x_offset: 0,
            y_offset: 0,
            x_step: 0,
            y_step: 0,
            num_steps: 0,
            step_timer: 0.0,
            step_count: 0.0,
            step_increment: 1.0,
        }
    }

    pub fn render(&mut self, position: Vec3, angle: f32, scale: Vec3, delta_time: f32) {
        match self.animation_type {
            SpriteAnimationType::None => {}
            SpriteAnimationType::Once => {}
            SpriteAnimationType::Cycle => {}
            SpriteAnimationType::BackAndForth => self.update_back_and_forth(delta_time),
        }

        self.shader.setVec2("offset", &vec2(32.0 * self.step_count, 0.0));

        self.mesh.render(&self.shader, position, angle, scale);
    }

    fn update_back_and_forth(&mut self, delta_time: f32) {
        if self.step_timer <= 0.0 {
            self.step_timer = 0.2;
            self.step_count += self.step_increment;
            if self.step_count > 2.0 {
                self.step_count = 1.0;
                self.step_increment = -1.0;
            }
            if self.step_count < 0.0 {
                self.step_count = 1.0;
                self.step_increment = 1.0;
            }
        }
        self.step_timer -= delta_time;
    }
}
