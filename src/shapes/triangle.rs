use crate::core::shader::Shader;
use crate::core::SIZE_OF_FLOAT;
use glad_gl::gl;
use glad_gl::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use glam::{vec3, Mat4, Vec3};
use std::ptr;

pub struct Triangle {
    VAO: GLuint,
    VBO: GLuint,
}

impl Triangle {
    pub fn new() -> Self {
        let mut VAO: GLuint = 0;
        let mut VBO: GLuint = 0;

        #[rustfmt::skip]
            let vertices: [f32; 9] = [
            -1.0,  0.6,  0.0,
            1.0,  0.0,  0.0,
            -1.0, -0.6,  0.0
        ];

        unsafe {
            gl::GenVertexArrays(1, &mut VAO);
            gl::GenBuffers(1, &mut VBO);
            gl::BindVertexArray(VAO);

            gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * SIZE_OF_FLOAT) as GLsizeiptr,
                vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (3 * SIZE_OF_FLOAT) as GLsizei, ptr::null());
            gl::EnableVertexAttribArray(0);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Triangle { VAO, VBO }
    }

    pub fn render(&self, shader: &Shader, position: Vec3, angle: f32, scale: Vec3, color: &Vec3) {
        let mut model_transform = Mat4::from_translation(position);
        model_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), angle.to_radians());
        model_transform *= Mat4::from_scale(scale);

        shader.use_shader();
        shader.setMat4("model", &model_transform);
        shader.setVec3("color", color);

        unsafe {
            gl::BindVertexArray(self.VAO);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }
    }
}
