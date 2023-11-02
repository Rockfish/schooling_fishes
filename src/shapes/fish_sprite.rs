use crate::core::mesh::{Color, Mesh, Vertex};
use crate::core::shader::Shader;
use crate::core::sprite_model::{SpriteAnimationType, SpriteModel};
use crate::core::texture::{Texture, TextureConfig, TextureFilter, TextureType};
use glam::{vec2, vec3, Vec3};
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;

pub struct FishSprite(SpriteModel);

impl FishSprite {
    pub fn new_fish_mesh(texture: &Rc<Texture>) -> Mesh {
        let verts = vec![
            Vertex::new(vec3(-1.0, -2.0, 0.0), vec2(8.0, 2.0), Color::white()),
            Vertex::new(vec3(1.0, -2.0, 0.0), vec2(24.0, 2.0), Color::white()),
            Vertex::new(vec3(-1.0, 2.0, 0.0), vec2(8.0, 31.0), Color::white()),
            Vertex::new(vec3(1.0, 2.0, 0.0), vec2(24.0, 31.0), Color::white()),
        ];

        let indices = vec![
            0, 1, 2, //1, 2, 3,
            1, 3, 2, //  2, 4, 3
        ];

        Mesh::new(verts, indices, texture)
    }

    pub fn new_sprite_model(tile_shader: Rc<Shader>) -> SpriteModel {
        let file_tile_map = Rc::new(
            Texture::new(
                PathBuf::from("assets/images/fish_3.png"),
                &TextureConfig {
                    flip_v: true,
                    gamma_correction: false,
                    filter: TextureFilter::Linear,
                    texture_type: TextureType::Diffuse,
                },
            )
            .unwrap(),
        );

        let fish_mesh = FishSprite::new_fish_mesh(&file_tile_map);

        let fish_model = SpriteModel {
            name: Rc::from("Fish"),
            animation_type: SpriteAnimationType::BackAndForth,
            shader: tile_shader.clone(),
            mesh: Rc::new(fish_mesh),
            x_offset: 0,
            y_offset: 0,
            x_step: 0,
            y_step: 0,
            num_steps: 0,
            step_timer: 0.0,
            step_count: 0.0,
            step_increment: 1.0,
        };

        fish_model
    }
}
