use glam::{vec3, Mat4, Vec3};
use small_gl_core::gl;
use small_gl_core::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use small_gl_core::shader::Shader;
use small_gl_core::texture::Texture;
use small_gl_core::SIZE_OF_FLOAT;
use std::ptr;
use std::rc::Rc;

pub struct Plane {
    VAO: GLuint,
    VBO: GLuint,
    texture: Rc<Texture>,
}

impl Plane {
    pub fn new(texture: Rc<Texture>) -> Self {
        let mut VAO: GLuint = 0;
        let mut VBO: GLuint = 0;

        // Drawing as triangles.
        // #[rustfmt::skip]
        // let vertices: [f32; 30] = [
        //     // first
        //     -1.0,  -1.0,  0.0,  0.0, 0.0,
        //      1.0,  -1.0,  0.0,  1.0, 0.0,
        //     -1.0,   1.0,  0.0,  0.0, 1.0,
        //     // second
        //      1.0,  -1.0,  0.0,  1.0, 0.0,
        //      1.0,   1.0,  0.0,  1.0, 1.0,
        //     -1.0,   1.0,  0.0,  0.0, 1.0,
        // ];

        // XZ plane
        #[rustfmt::skip]
            let vertices: [f32; 30] = [
            // first
            0.0, 0.0, 0.0,   0.0, 0.0,
            1.0, 0.0, 0.0,   1.0, 0.0,
            0.0, 0.0, 1.0,   0.0, 1.0,
            // second
             1.0, 0.0, 0.0,   1.0, 0.0,
             1.0, 0.0, 1.0,   1.0, 1.0,
             0.0, 0.0, 1.0,   0.0, 1.0,
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

            // 0: position
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, ptr::null());
            gl::EnableVertexAttribArray(0);

            // 1: texture
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * SIZE_OF_FLOAT) as GLsizei,
                (3 * SIZE_OF_FLOAT) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(1);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        Plane { VAO, VBO, texture }
    }

    pub fn render(&self, shader: &Shader, position: Vec3, angle: f32, scale: Vec3) {
        let mut model_transform = Mat4::from_translation(position);
        model_transform *= Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), angle.to_radians());
        model_transform *= Mat4::from_scale(scale);

        shader.set_mat4("model", &model_transform);
        shader.set_int("texture_diffuse1", 0);

        unsafe {
            gl::BindVertexArray(self.VAO);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture.id);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }
}
