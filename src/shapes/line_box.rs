use crate::core::shader::Shader;
use crate::core::SIZE_OF_FLOAT;
use glad_gl::gl;
use glad_gl::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use glam::{Mat4, Vec3};
use std::ptr;

pub struct LineBox {
    line_VAO: GLuint,
    line_VBO: GLuint,
}

impl LineBox {
    pub fn new() -> Self {
        #[rustfmt::skip]
        let box_vertices: [f32; 12] = [
             0.0, 0.0, 0.0,
             1.0, 0.0, 0.0,
             1.0, 1.0, 0.0,
             0.0, 1.0, 0.0,
        ];

        let mut line_VAO: GLuint = 0;
        let mut line_VBO: GLuint = 0;

        unsafe {
            // box_lines
            gl::GenVertexArrays(1, &mut line_VAO);
            gl::GenBuffers(1, &mut line_VBO);
            gl::BindVertexArray(line_VAO);

            gl::BindBuffer(gl::ARRAY_BUFFER, line_VBO);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (box_vertices.len() * SIZE_OF_FLOAT) as GLsizeiptr,
                box_vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (3 * SIZE_OF_FLOAT) as GLsizei, ptr::null());
            gl::EnableVertexAttribArray(0);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        LineBox { line_VAO, line_VBO }
    }

    pub fn render(&self, shader: &Shader, position: Vec3, scale: Vec3, color: &Vec3) {
        unsafe {
            // shader.use_shader();
            let mut model_transform = Mat4::from_translation(position);
            model_transform *= Mat4::from_scale(scale);
            shader.setMat4("model", &model_transform);
            shader.setVec3("color", &color);

            gl::BindVertexArray(self.line_VAO);
            gl::DrawArrays(gl::LINE_LOOP, 0, 4);
        }
    }
}
