use std::mem;
use crate::support::error::Error;
use glam::{Mat4, Vec2, Vec3, vec3};
use std::path::Path;
use glad_gl::gl;
use glad_gl::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use crate::support::model::{FlipV, Gamma};
use crate::support::shader::Shader;
use crate::support::texture::load_texture;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, packed)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Color {r: 1.0, g: 1.0, b: 1.0, a: 1.0}
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Vertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
    pub color: Color,
}

impl Vertex {
    pub fn new(position: Vec3, tex_coords: Vec2, color: Color) -> Vertex {
        Vertex {
            position,
            tex_coords,
            color,
        }
    }
}
impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Default::default(),
            tex_coords: Default::default(),
            color: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub id: u32,
    pub texture_type: String, // texture uniform name
    pub texture_path: String,
}

impl Texture {
    pub fn new(texture_path: impl Into<String>, texture_type: impl Into<String>, flip_v: FlipV, gamma: Gamma) -> Result<Texture, Error> {
        let texture_path= texture_path.into();
        let id = load_texture(&Path::new(&texture_path), gamma.0, flip_v.0)?;
        let texture = Texture {
            id,
            texture_type: texture_type.into(),
            texture_path,
        };
        Ok(texture)
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub texture: Texture,
    pub VAO: u32,
    pub VBO: u32,
    pub EBO: u32,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, texture: &Texture) -> Mesh {
        let mut VAO: GLuint = 0;
        let mut VBO: GLuint = 0;
        let mut EBO: GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut VAO);
            gl::GenBuffers(1, &mut VBO);
            gl::GenBuffers(1, &mut EBO);

            // load vertex data into vertex buffers
            gl::BindVertexArray(VAO);
            gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<Vertex>()) as GLsizeiptr,
                vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // load index data into element buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, EBO);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * mem::size_of::<u32>()) as GLsizeiptr,
                indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // set the vertex attribute pointers vertex Positions
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                0 as *const GLvoid);

            // vertex texture coordinates
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, tex_coords) as *const GLvoid,
            );

            // vertex color
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as GLsizei,
                mem::offset_of!(Vertex, color) as *const GLvoid,
            );

            gl::BindVertexArray(0);
        }

        Mesh {
            vertices,
            indices,
            texture: texture.clone(),
            VAO,
            VBO,
            EBO
        }
    }

    // pub fn draw(&self, shader: &Shader) {
    //     shader.setInt("texture1", 1);
    //
    //     unsafe {
    //         gl::BindVertexArray(self.VAO);
    //         gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, 0 as *const GLvoid);
    //         gl::BindVertexArray(0);
    //     }
    // }

    pub fn render(&self, shader: &Shader, position: Vec3, angle: f32, scale: Vec3) {
        let mut model_transform = Mat4::from_translation(position);
        model_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), angle.to_radians());
        model_transform *= Mat4::from_scale(scale);

        shader.setMat4("model", &model_transform);
        shader.setInt("texture1", 0);

        unsafe {
            gl::BindVertexArray(self.VAO);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture.id);
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, 0 as *const GLvoid);
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.VAO);
            gl::DeleteBuffers(1, &self.VBO);
            gl::DeleteBuffers(1, &self.EBO);
        }
    }
}
