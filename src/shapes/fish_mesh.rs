use crate::support::mesh::{Color, Mesh, Texture, Vertex};
use glam::{vec3, vec2};

pub fn new_fish_mesh(texture: &Texture) -> Mesh {
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
        Vertex::new(vec3(-1.0, -2.0, 0.0), vec2(8.0, 2.0), Color::default()),
        Vertex::new(vec3(1.0, -2.0, 0.0), vec2(24.0, 2.0), Color::default()),
        Vertex::new(vec3(-1.0, 2.0, 0.0), vec2(8.0, 30.0), Color::default()),
        Vertex::new(vec3(1.0, 2.0, 0.0), vec2(24.0, 30.0), Color::default()),
    ];

    let indices= vec![
        1, 2, 3,
        2, 4, 3
    ];

    Mesh::new(verts, indices, texture)
}
