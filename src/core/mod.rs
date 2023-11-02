pub mod assimp_scene;
pub mod camera;
pub mod error;
pub mod macros;
pub mod mesh;
pub mod model;
pub mod model_mesh;
pub mod shader;
pub mod sprite_model;
pub mod texture;

type ShaderId = u32;

pub const SIZE_OF_FLOAT: usize = std::mem::size_of::<f32>();
