use crate::core::mesh::Mesh;
use crate::core::shader::Shader;
use glam::{vec2, Vec2, Vec3};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum SpriteAnimationType {
    None,
    Once,
    Cycle,
    BackAndForth,
}

/*
   A sprite is a glyph or series of glyphs contained in a texture.
*/

#[derive(Debug, Clone)]
pub struct SpriteData {
    pub animation_type: SpriteAnimationType,
    pub texture_width: f32,
    pub texture_height: f32,
    pub offset: Vec2, // position of the glyph in the texture
    pub x_step: f32,  // horizontal offset between glyphs for the same sprite
    pub y_step: f32,  // vertical offset between glyphs for the same sprite
    pub num_steps: u32,
    pub step_timer: f32,
    pub step_count: f32,
    pub step_increment: f32,
}

#[derive(Debug, Clone)]
pub struct SpriteModel {
    pub name: Rc<str>,
    pub shader: Rc<Shader>,
    pub mesh: Rc<Mesh>,
    pub sprite_data: SpriteData,
}

impl SpriteModel {
    pub fn new(name: &Rc<str>, shader: &Rc<Shader>, mesh: &Rc<Mesh>, sprite_data: SpriteData) -> Self {
        SpriteModel {
            name: name.clone(),
            shader: shader.clone(),
            mesh: mesh.clone(),
            sprite_data,
        }
    }

    pub fn render(&mut self, position: Vec3, angle: f32, scale: Vec3, delta_time: f32) {
        match self.sprite_data.animation_type {
            SpriteAnimationType::None => {}
            SpriteAnimationType::Once => {}
            SpriteAnimationType::Cycle => {}
            SpriteAnimationType::BackAndForth => self.update_back_and_forth(delta_time),
        }

        self.shader.setVec2(
            "offset",
            &vec2(
                self.sprite_data.offset.x + self.sprite_data.x_step * self.sprite_data.step_count,
                self.sprite_data.offset.y + self.sprite_data.y_step * self.sprite_data.step_count,
            ),
        );

        self.shader
            .setVec2("tex_size", &vec2(self.sprite_data.texture_width, self.sprite_data.texture_height));

        self.mesh.render(&self.shader, position, angle, scale);
    }

    fn update_back_and_forth(&mut self, delta_time: f32) {
        if self.sprite_data.step_timer <= 0.0 {
            self.sprite_data.step_timer = 0.2;
            self.sprite_data.step_count += self.sprite_data.step_increment;
            if self.sprite_data.step_count > 2.0 {
                self.sprite_data.step_count = 1.0;
                self.sprite_data.step_increment = -1.0;
            }
            if self.sprite_data.step_count < 0.0 {
                self.sprite_data.step_count = 1.0;
                self.sprite_data.step_increment = 1.0;
            }
        }
        self.sprite_data.step_timer -= delta_time;
    }
}
