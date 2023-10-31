use crate::core::shader::Shader;
use crate::core::texture::Texture;
use glad_gl::gl;
use glad_gl::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use glam::{vec3, Mat4, Vec2, Vec3};
use std::mem;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, packed)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub(crate) fn white() -> Self {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
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
            color: Color::white(),
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub texture: Rc<Texture>,
    pub VAO: u32,
    pub VBO: u32,
    pub EBO: u32,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, texture: &Rc<Texture>) -> Mesh {
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

            // vertex positions
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, mem::size_of::<Vertex>() as GLsizei, 0 as *const GLvoid);

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
            EBO,
        }
    }

    pub fn render(&self, shader: &Shader, position: Vec3, angle: f32, scale: Vec3) {
        let mut model_transform = Mat4::from_translation(position);
        model_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), angle.to_radians());
        model_transform *= Mat4::from_scale(scale);
        shader.setMat4("model", &model_transform);

        let texture_location = 0;
        shader.setInt("texture_diffuse1", texture_location as i32);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + texture_location);
            gl::BindVertexArray(self.VAO);
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
