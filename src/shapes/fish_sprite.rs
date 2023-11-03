use std::collections::HashMap;
use crate::core::mesh::{Color, Mesh, Vertex};
use crate::core::shader::Shader;
use crate::core::sprite_model::{SpriteAnimationType, SpriteData, SpriteModel};
use crate::core::texture::{Texture, TextureConfig, TextureFilter, TextureType};
use glam::{vec2, vec3};
use std::path::PathBuf;
use std::rc::Rc;

pub struct FishSprite;

impl FishSprite {

    pub fn new_fish_mesh(texture: &Rc<Texture>, flip_to_xz: bool) -> Mesh {

        let verts = vec![
            Vertex::new(vec3(-1.0, -2.0, 0.0), vec2(0.0, 0.0), Color::white()),
            Vertex::new(vec3(1.0, -2.0, 0.0), vec2(16.0, 0.0), Color::white()),
            Vertex::new(vec3(-1.0, 2.0, 0.0), vec2(0.0, 29.0), Color::white()),
            Vertex::new(vec3(1.0, 2.0, 0.0), vec2(16.0, 29.0), Color::white()),
        ];

        let indices = vec![
            0, 1, 2,
            1, 3, 2,
        ];

        Mesh::new(verts, indices, texture, flip_to_xz)
    }

    pub fn new_sprite_model(tile_shader: Rc<Shader>, flip_to_xz: bool) -> SpriteModel {

        let tile_texture = Rc::new(
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

        let fish_mesh = FishSprite::new_fish_mesh(&tile_texture, flip_to_xz);

        // fish positions by color
        let mut offsets = HashMap::new();
        offsets.insert("gold", vec2(8.0, 2.0));
        offsets.insert("grey", vec2(104.0, 2.0));
        offsets.insert("blue", vec2(8.0, 130.0));


        let fish_data = SpriteData {
            animation_type: SpriteAnimationType::BackAndForth,
            texture_width: fish_mesh.texture.width as f32,
            texture_height: fish_mesh.texture.height as f32,
            offset: offsets["gold"],
            x_step: 32.0,
            y_step: 0.0,
            num_steps: 0,
            step_timer: 0.0,
            step_count: 0.0,
            step_increment: 1.0,
        };

        let fish_model = SpriteModel {
            name: Rc::from("Fish"),
            shader: tile_shader.clone(),
            mesh: Rc::new(fish_mesh),
            sprite_data: fish_data,
        };

        fish_model
    }
}
