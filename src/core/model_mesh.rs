// #![allow(unused_variables)]
// #![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]

use crate::core::shader::Shader;
use crate::core::texture::{Texture, TextureType};
use glad_gl::gl;
use glad_gl::gl::{GLsizei, GLsizeiptr, GLvoid};
use glam::u32;
use glam::*;
use std::mem;
use std::ops::Add;
use std::rc::Rc;

const MAX_BONE_INFLUENCE: usize = 4;
const OFFSET_OF_NORMAL: usize = mem::offset_of!(ModelVertex, normal);
const OFFSET_OF_TEXCOORDS: usize = mem::offset_of!(ModelVertex, tex_coords);
const OFFSET_OF_TANGENT: usize = mem::offset_of!(ModelVertex, tangent);
const OFFSET_OF_BITANGENT: usize = mem::offset_of!(ModelVertex, bi_tangent);
const OFFSET_OF_BONE_IDS: usize = mem::offset_of!(ModelVertex, bone_ids);
const OFFSET_OF_WEIGHTS: usize = mem::offset_of!(ModelVertex, bone_weights);

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct ModelVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec2,
    pub tangent: Vec3,
    pub bi_tangent: Vec3,
    pub bone_ids: [i32; MAX_BONE_INFLUENCE],
    pub bone_weights: [f32; MAX_BONE_INFLUENCE],
}

impl ModelVertex {
    pub fn new() -> ModelVertex {
        ModelVertex {
            position: Vec3::default(),
            normal: Vec3::default(),
            tex_coords: Vec2::default(),
            tangent: Vec3::default(),
            bi_tangent: Vec3::default(),
            bone_ids: [0; MAX_BONE_INFLUENCE],
            bone_weights: [0.0; MAX_BONE_INFLUENCE],
        }
    }
}
impl Default for ModelVertex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ModelMesh {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Rc<Texture>>,
    pub VAO: u32,
    pub VBO: u32,
    pub EBO: u32,
}

impl ModelMesh {
    pub fn new(vertices: Vec<ModelVertex>, indices: Vec<u32>, textures: Vec<Rc<Texture>>) -> ModelMesh {
        let mut mesh = ModelMesh {
            vertices,
            indices,
            textures,
            VAO: 0,
            VBO: 0,
            EBO: 0,
        };
        mesh.setupMesh();
        mesh
    }

    pub fn render(&self, shader: &Rc<Shader>) {
        // bind appropriate textures
        let mut diffuse_count: u32 = 0;
        let mut specular_count: u32 = 0;
        let mut normal_count: u32 = 0;
        let mut height_count: u32 = 0;

        unsafe {
            // set the location and binding for all the textures
            for (texture_unit, texture) in self.textures.iter().enumerate() {
                // active proper texture unit before binding
                gl::ActiveTexture(gl::TEXTURE0 + texture_unit as u32);

                // retrieve texture number (the N in diffuse_textureN)
                let num = match texture.texture_type {
                    TextureType::Diffuse => {
                        diffuse_count += 1;
                        diffuse_count
                    }
                    TextureType::Specular => {
                        specular_count += 1;
                        specular_count
                    }
                    TextureType::Normals => {
                        normal_count += 1;
                        normal_count
                    }
                    TextureType::Height => {
                        height_count += 1;
                        height_count
                    }
                    _ => todo!(),
                };

                // now set the sampler to the correct texture unit (location)
                let texture_name = texture.texture_type.to_string().clone().add(&num.to_string());
                shader.setInt(&texture_name, texture_unit as i32);

                gl::BindTexture(gl::TEXTURE_2D, texture.id);
            }

            gl::BindVertexArray(self.VAO);
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, 0 as *const GLvoid);
            gl::BindVertexArray(0);
        }
    }

    fn setupMesh(&mut self) {
        unsafe {
            gl::GenVertexArrays(1, &mut self.VAO);
            gl::GenBuffers(1, &mut self.VBO);
            gl::GenBuffers(1, &mut self.EBO);

            // load vertex data into vertex buffers
            gl::BindVertexArray(self.VAO);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.VBO);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * mem::size_of::<ModelVertex>()) as GLsizeiptr,
                self.vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // load index data into element buffer
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.EBO);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<u32>()) as GLsizeiptr,
                self.indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // set the vertex attribute pointers vertex Positions
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                0 as *const GLvoid,
            );

            // vertex normals
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_NORMAL) as *const GLvoid,
            );

            // vertex texture coordinates
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_TEXCOORDS) as *const GLvoid,
            );

            // vertex tangent
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(
                3,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_TANGENT) as *const GLvoid,
            );

            // vertex bitangent
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(
                4,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_BITANGENT) as *const GLvoid,
            );

            // bone ids
            gl::EnableVertexAttribArray(5);
            gl::VertexAttribPointer(
                5,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_BONE_IDS) as *const GLvoid,
            );

            // weights
            gl::EnableVertexAttribArray(6);
            gl::VertexAttribPointer(
                6,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<ModelVertex>() as GLsizei,
                (OFFSET_OF_WEIGHTS) as *const GLvoid,
            );

            gl::BindVertexArray(0);
        }
    }
}

impl Drop for ModelMesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.VAO);
            gl::DeleteBuffers(1, &self.VBO);
            gl::DeleteBuffers(1, &self.EBO);
        }
    }
}

pub fn print_model_mesh(mesh: &ModelMesh) {
    println!("mesh: {:#?}", mesh);

    println!("size vertex: {}", mem::size_of::<ModelVertex>());
    println!("OFFSET_OF_NORMAL: {}", mem::offset_of!(ModelVertex, normal));
    println!("OFFSET_OF_TEXCOORDS: {}", mem::offset_of!(ModelVertex, tex_coords));
    println!("OFFSET_OF_TANGENT: {}", mem::offset_of!(ModelVertex, tangent));
    println!("OFFSET_OF_BITANGENT: {}", mem::offset_of!(ModelVertex, bi_tangent));
    println!("OFFSET_OF_BONE_IDS: {}", mem::offset_of!(ModelVertex, bone_ids));
    println!("OFFSET_OF_WEIGHTS: {}", mem::offset_of!(ModelVertex, bone_weights));

    println!("size of Vec3: {}", mem::size_of::<Vec3>());
    println!("size of Vec2: {}", mem::size_of::<Vec2>());
    println!("size of [i32;4]: {}", mem::size_of::<[i32; MAX_BONE_INFLUENCE]>());
    println!("size of [f32;4]: {}", mem::size_of::<[f32; MAX_BONE_INFLUENCE]>());

    println!(
        "size of vertex parts: {}",
        mem::size_of::<Vec3>() * 4
            + mem::size_of::<Vec2>()
            + mem::size_of::<[i32; MAX_BONE_INFLUENCE]>()
            + mem::size_of::<[f32; MAX_BONE_INFLUENCE]>()
    );
}
