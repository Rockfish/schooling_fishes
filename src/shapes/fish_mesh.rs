use crate::core::mesh::{Color, Mesh, Vertex};
use crate::core::texture::Texture;
use glam::{vec2, vec3};
use std::rc::Rc;

pub fn new_fish_mesh(texture: &Rc<Texture>) -> Mesh {
    // let x = 1.0f32;
    // let y = 1.0f32;

    // Drawing as triangles.
    #[rustfmt::skip]
    // let vertices: [f32; 12] = [
    //     -1.0,  -2.0,  0.0,
    //      1.0,  -2.0,  0.0,
    //     -1.0,   2.0,  0.0,
    //      1.0,   2.0,  0.0,
    // ];
    //
    // #[rustfmt::skip]
    // let tex_coords: [f32; 8] = [
    //      8.0,  2.0,
    //     24.0,  2.0,
    //      8.0, 30.0,
    //     24.0, 30.0,
    // ];
    //
    // let color = Color::default();

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
