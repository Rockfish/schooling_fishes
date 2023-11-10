use glam::{Vec2, vec2, vec3, Vec3};
use crate::core::mesh::{Color, Mesh, Vertex};

pub fn build_vertexes_and_indices(width: u32, height: u32, color: Color) -> (Vec<Vertex>, Vec<u32>) {
    let (verts, indices, uvs) = build_verts_and_indices_uvs(width, height);

    let mut vertices = Vec::with_capacity(verts.len());


    for i in 0..verts.len() {

        let x = verts[i].x;
        let z = verts[i].z;
        let y = 5.0 * ((x/width as f32)*30.0).cos() - 5.0 *((z/height as f32) * 30.0).cos();
        let vert = vec3(x, y, z);

        let vertex = Vertex {
            position: vert,
            tex_coords: uvs[i],
            color: color.clone(),
        };
        vertices.push(vertex);
    }

    (vertices, indices)
}

// build mesh in the XZ plane
pub fn build_verts_and_indices_uvs(width: u32, height: u32) -> (Vec<Vec3>, Vec<u32>, Vec<Vec2>) {

    let mut vertices: Vec<Vec3> = Vec::with_capacity((width * height) as usize);
    let mut indices : Vec<u32> = Vec::with_capacity(((width - 1) * 2 * (height - 1)) as usize);
    let mut uvs : Vec<Vec2> = Vec::with_capacity((width * height) as usize);

    let mut i = 0;
    for z in 0..height {
        for x in 0..width {
            let vert = vec3(x as f32, 0.0, z as f32);
            // let vert = vec3(x as f32,z as f32, 0.0);
            vertices.push(vert);

            let uv = vec2(x as f32/(width as f32- 1f32), z as f32 /(height as f32 - 1f32));
            uvs.push(uv);

            if x < width - 1 && z < height - 1 {
                indices.push(i);
                indices.push(i + 1);
                indices.push(i + width);
                indices.push(i + 1);
                indices.push(i + width + 1);
                indices.push(i + width);
                i += 1;
            }
        }
        i += 1;
    }

    (vertices, indices, uvs)
}

#[cfg(test)]
mod tests {
    use crate::shapes::mesh_plane::build_verts_and_indices_uvs;

    #[test]
    pub fn test_verts_and_indices() {
        let width = 4u32;
        let height = 4u32;

        let (verts, indices, uvs) = build_verts_and_indices_uvs(width, height);

        for i in 0..verts.len() {
            println!("{} - {:?}  {:?}", i+1, verts[i], uvs[i]);
        }

        let mut c = 0;
        for i in 0..indices.len() / 3 {
            println!("{} - {}, {}, {}", i+1, indices[c], indices[c+1], indices[c+2]);
            c += 3;
        }
        println!("num indices (width -1)*2 * (height -1): {}", (width - 1) * 2 * (height - 1));
    }

}
