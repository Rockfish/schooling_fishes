// #![allow(unused_variables)]
// #![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]

use crate::core::texture::{Texture, TextureType};
use crate::core::ShaderId;
use glad_gl::gl;
use glad_gl::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use glam::u32;
use glam::*;
use std::ffi::CString;
use std::mem;
use std::ops::Add;
use std::rc::Rc;

const MAX_BONE_INFLUENCE: usize = 4;
const OFFSET_OF_NORMAL: usize = mem::offset_of!(ModelVertex, Normal);
const OFFSET_OF_TEXCOORDS: usize = mem::offset_of!(ModelVertex, TexCoords);
const OFFSET_OF_TANGENT: usize = mem::offset_of!(ModelVertex, Tangent);
const OFFSET_OF_BITANGENT: usize = mem::offset_of!(ModelVertex, Bitangent);
const OFFSET_OF_BONE_IDS: usize = mem::offset_of!(ModelVertex, m_BoneIDs);
const OFFSET_OF_WEIGHTS: usize = mem::offset_of!(ModelVertex, m_Weights);

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct ModelVertex {
    pub Position: Vec3,
    pub Normal: Vec3,
    pub TexCoords: Vec2,
    pub Tangent: Vec3,
    pub Bitangent: Vec3,
    pub m_BoneIDs: [i32; MAX_BONE_INFLUENCE],
    pub m_Weights: [f32; MAX_BONE_INFLUENCE],
}

impl ModelVertex {
    pub fn new() -> ModelVertex {
        ModelVertex {
            Position: Vec3::default(),
            Normal: Vec3::default(),
            TexCoords: Vec2::default(),
            Tangent: Vec3::default(),
            Bitangent: Vec3::default(),
            m_BoneIDs: [0; MAX_BONE_INFLUENCE],
            m_Weights: [0.0; MAX_BONE_INFLUENCE],
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

    pub fn Draw(&self, shader_id: ShaderId) {
        // bind appropriate textures
        let mut diffuseNr: u32 = 0;
        let mut specularNr: u32 = 0;
        let mut normalNr: u32 = 0;
        let mut heightNr: u32 = 0;

        unsafe {
            for (i, texture) in self.textures.iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + i as u32); // active proper texture unit before binding

                // retrieve texture number (the N in diffuse_textureN)
                let num = match texture.texture_type {
                    TextureType::Diffuse => {
                        diffuseNr += 1;
                        diffuseNr
                    }
                    TextureType::Specular => {
                        specularNr += 1;
                        specularNr
                    }
                    TextureType::Normals => {
                        normalNr += 1;
                        normalNr
                    }
                    TextureType::Height => {
                        heightNr += 1;
                        heightNr
                    }
                    _ => todo!(),
                };

                // now set the sampler to the correct texture unit
                let name = texture.texture_type.to_string().clone().add(&num.to_string());
                let c_name = CString::new(name).unwrap();

                gl::Uniform1i(gl::GetUniformLocation(shader_id, c_name.as_ptr()), i as i32);
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
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, mem::size_of::<ModelVertex>() as GLsizei, 0 as *const GLvoid);

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

    pub fn debug(&self) {
        println!("mesh: {:#?}", self);

        println!("size vertex: {}", mem::size_of::<ModelVertex>());
        println!("OFFSET_OF_NORMAL: {}", mem::offset_of!(ModelVertex, Normal));
        println!("OFFSET_OF_TEXCOORDS: {}", mem::offset_of!(ModelVertex, TexCoords));
        println!("OFFSET_OF_TANGENT: {}", mem::offset_of!(ModelVertex, Tangent));
        println!("OFFSET_OF_BITANGENT: {}", mem::offset_of!(ModelVertex, Bitangent));
        println!("OFFSET_OF_BONE_IDS: {}", mem::offset_of!(ModelVertex, m_BoneIDs));
        println!("OFFSET_OF_WEIGHTS: {}", mem::offset_of!(ModelVertex, m_Weights));

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
}
