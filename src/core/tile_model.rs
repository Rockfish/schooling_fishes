use std::rc::Rc;
use glam::{vec2, Vec3};
use crate::core::mesh::Mesh;
use crate::core::shader::Shader;

pub enum TileAnimationType {
    None,
    Once,
    Cycle,
    BackAndForth,
}

pub struct TileModel {
    pub name: Rc<str>,
    pub animation_type: TileAnimationType,
    pub shader: Rc<Shader>,
    pub mesh: Rc<Mesh>,
    pub x_offset: u32,
    pub y_offset: u32,
    pub x_step: u32,
    pub y_step: u32,
    pub num_steps: u32,
    pub step_time: f32,
    pub step_count: f32,
    pub step_increment: f32,
}

impl TileModel {

    pub fn render(&mut self, position: Vec3, angle: f32, scale: Vec3, delta_time: f32) {
    // pub fn render(&mut self, projection_view: Mat4, position: Vec3, angle: f32, scale: Vec3, delta_time: f32) {
        match self.animation_type {
            TileAnimationType::None => {}
            TileAnimationType::Once => {}
            TileAnimationType::Cycle => {}
            TileAnimationType::BackAndForth => self.update_back_and_forth(delta_time),
        }

        self.shader.setVec2("offset", &vec2(32.0 * self.step_count, 0.0));

        self.mesh.render(&self.shader, position, angle, scale);
    }

    fn update_back_and_forth(&mut self, delta_time: f32) {
        if self.step_time <= 0.0 {
            self.step_time = 50.1;  // set to a high number for now because many fish, one model.
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
        self.step_time -= delta_time;
    }
}