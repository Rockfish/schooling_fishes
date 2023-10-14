#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(unused_variables)]

use crate::aiscene::*;
use crate::mesh::{Mesh, Texture, Vertex};
use crate::ShaderId;
use glad_gl::gl;
use glad_gl::gl::{GLint, GLsizei, GLuint, GLvoid};
use glam::*;
use image::ColorType;
use russimp::scene::*;
use russimp::sys::*;
use std::os::raw::c_uint;
use std::path::{Path, PathBuf};
use std::ptr::*;

// model data
#[derive(Debug)]
pub struct Model {
    // stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    pub textures_loaded: Vec<Texture>,
    pub meshes: Vec<Mesh>,
    pub directory: String,
    pub gammaCorrection: bool,
    pub flipv: bool,
}

pub struct Gamma(pub bool);
pub struct FlipV(pub bool);

impl Model {
    pub fn new(path: &str, gamma: Gamma, flipv: FlipV) -> Model {
        let mut model = Model {
            textures_loaded: vec![],
            meshes: vec![],
            directory: "".to_string(),
            gammaCorrection: gamma.0,
            flipv: flipv.0,
        };
        model.load_model(path);
        model
    }

    pub fn Draw(&self, shader_id: ShaderId) {
        for mesh in &self.meshes {
            mesh.Draw(shader_id);
        }
    }

    // loads a model with supported ASSIMP extensions from file and stores the resulting meshes in the meshes vector.
    fn load_model(&mut self, path: &str) {
        let scene = AiScene::from_file(
            path,
            vec![
                PostProcess::Triangulate,
                PostProcess::GenerateSmoothNormals,
                PostProcess::FlipUVs,
                PostProcess::CalculateTangentSpace,
                // PostProcess::JoinIdenticalVertices,
                // PostProcess::SortByPrimitiveType,
                // PostProcess::EmbedTextures,
            ],
        );

        match scene {
            Ok(scene) => {
                self.directory = Path::new(path).parent().expect("path error").to_str().unwrap().to_string();

                if let Some(aiscene) = scene.assimp_scene {
                    self.process_node(aiscene.mRootNode, aiscene);
                }
            }
            Err(err) => panic!("{}", err),
        }
        // println!("Model:\n{:#?}", self);
    }

    fn process_node(&mut self, node: *mut aiNode, scene: &aiScene) {
        // process each mesh located at the current node
        // println!("{:?}", unsafe { (*node).mName });

        let slice = slice_from_raw_parts(scene.mMeshes, scene.mNumMeshes as usize);
        let ai_meshes = unsafe { slice.as_ref() }.unwrap();

        for i in 0..ai_meshes.len() {
            let mesh = self.process_mesh(ai_meshes[i], scene);
            self.meshes.push(mesh);
        }

        // Process childern nodes
        let slice = unsafe { slice_from_raw_parts((*node).mChildren, (*node).mNumChildren as usize) };

        if let Some(child_nodes) = unsafe { slice.as_ref() } {
            for i in 0..child_nodes.len() {
                // println!("{:#?}", unsafe { (*child_nodes[i]).mName });
                self.process_node(child_nodes[i], scene);
            }
        }
    }

    fn process_mesh(&mut self, scene_mesh: *mut aiMesh, scene: &aiScene) -> Mesh {
        let scene_mesh = unsafe { *scene_mesh };

        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        let mut textures: Vec<Texture> = vec![];

        let ai_vertices = get_vec_from_parts(scene_mesh.mVertices, scene_mesh.mNumVertices);
        let ai_normals = get_vec_from_parts(scene_mesh.mNormals, scene_mesh.mNumVertices);
        let ai_tangents = get_vec_from_parts(scene_mesh.mTangents, scene_mesh.mNumVertices);
        let ai_bitangents = get_vec_from_parts(scene_mesh.mBitangents, scene_mesh.mNumVertices);

        // a vertex can contain up to 8 different texture coordinates. We thus make the assumption that we won't
        // use models where a vertex can have multiple texture coordinates so we always take the first set (0).
        let texture_coords = if !scene_mesh.mTextureCoords.is_empty() {
            get_vec_from_parts(scene_mesh.mTextureCoords[0], ai_vertices.len() as u32)
        } else {
            vec![]
        };

        for i in 0..ai_vertices.len() {
            let mut vertex = Vertex::new();

            // positions
            vertex.Position = ai_vertices[i]; // Vec3 has Copy trait

            // normals
            if !ai_normals.is_empty() {
                vertex.Normal = ai_normals[i];
            }

            // texture coordinates
            if !texture_coords.is_empty() {
                // texture coordinates
                vertex.TexCoords = vec2(texture_coords[i].x, texture_coords[i].y);
                // tangent
                vertex.Tangent = ai_tangents[i];
                // bitangent
                vertex.Bitangent = ai_bitangents[i];
            } else {
                vertex.TexCoords = vec2(0.0, 0.0);
            }
            vertices.push(vertex);
        }
        // now walk through each of the mesh's faces (a face is a mesh its triangle) and retrieve the corresponding vertex indices.
        let ai_faces = unsafe { slice_from_raw_parts(scene_mesh.mFaces, scene_mesh.mNumFaces as usize).as_ref() }.unwrap();
        for i in 0..ai_faces.len() {
            let face = ai_faces[i];
            let ai_indices = unsafe { slice_from_raw_parts(face.mIndices, face.mNumIndices as usize).as_ref() }.unwrap();
            indices.extend(ai_indices.iter());
        }

        // process materials
        let ai_materials = unsafe { slice_from_raw_parts((*scene).mMaterials, (*scene).mNumMaterials as usize).as_ref() }.unwrap();
        let material_index = scene_mesh.mMaterialIndex as usize;
        let ai_material = ai_materials[material_index];

        // we assume a convention for sampler names in the shaders. Each diffuse texture should be named
        // as 'texture_diffuseN' where N is a sequential number ranging from 1 to MAX_SAMPLER_NUMBER.
        // Same applies to other texture as the following list summarizes:
        // diffuse: texture_diffuseN
        // specular: texture_specularN
        // normal: texture_normalN

        // 1. diffuse maps
        let diffuseMaps = self.loadMaterialTextures(ai_material, aiTextureType_DIFFUSE, "texture_diffuse");
        textures.extend(diffuseMaps);
        // 2. specular maps
        let specularMaps = self.loadMaterialTextures(ai_material, aiTextureType_SPECULAR, "texture_specular");
        textures.extend(specularMaps);
        // 3. normal maps
        let normalMaps = self.loadMaterialTextures(ai_material, aiTextureType_HEIGHT, "texture_normal");
        textures.extend(normalMaps);
        // 4. height maps
        let heightMaps = self.loadMaterialTextures(ai_material, aiTextureType_AMBIENT, "texture_height");
        textures.extend(heightMaps);

        let mesh = Mesh::new(vertices, indices, textures);
        mesh
    }

    fn loadMaterialTextures(&mut self, ai_material: *mut aiMaterial, ai_texture_type: c_uint, typeName: &str) -> Vec<Texture> {
        let mut textures: Vec<Texture> = vec![];

        let texture_count = unsafe { aiGetMaterialTextureCount(ai_material, ai_texture_type) };

        for i in 0..texture_count {
            let texture_file = get_material_texture_filename(ai_material, ai_texture_type, i as u32);
            if let Ok(filename) = texture_file {
                let loaded_texture = self.textures_loaded.iter().find(|t| t.path == filename);
                if let Some(texture) = loaded_texture {
                    textures.push(texture.clone());
                } else {
                    let mut filepath = PathBuf::from(&self.directory);
                    filepath.push(&filename);
                    let id = self.textureFromFile(&filepath);
                    let texture = Texture {
                        id,
                        texture_type: typeName.to_string(),
                        path: filename,
                    };
                    textures.push(texture.clone());
                    self.textures_loaded.push(texture);
                }
            }
        }
        textures
    }

    fn textureFromFile(&self, filepath: &Path) -> u32 {
        let mut texture_id: GLuint = 0;

        let img = image::open(filepath).expect("Texture failed to load");
        let (width, height) = (img.width() as GLsizei, img.height() as GLsizei);

        let color_type = img.color();

        let img = if self.flipv { img.flipv() } else { img };

        unsafe {
            let format = match color_type {
                ColorType::L8 => gl::RED,
                // ColorType::La8 => {}
                ColorType::Rgb8 => gl::RGB,
                ColorType::Rgba8 => gl::RGBA,
                // ColorType::L16 => {}
                // ColorType::La16 => {}
                // ColorType::Rgb16 => {}
                // ColorType::Rgba16 => {}
                // ColorType::Rgb32F => {}
                // ColorType::Rgba32F => {}
                _ => panic!("no mapping for color type"),
            };

            let data = match color_type {
                ColorType::L8 => img.into_rgb8().into_raw(),
                // ColorType::La8 => {}
                ColorType::Rgb8 => img.into_rgb8().into_raw(),
                ColorType::Rgba8 => img.into_rgba8().into_raw(),
                // ColorType::L16 => {}
                // ColorType::La16 => {}
                // ColorType::Rgb16 => {}
                // ColorType::Rgba16 => {}
                // ColorType::Rgb32F => {}
                // ColorType::Rgba32F => {}
                _ => panic!("no mapping for color type"),
            };

            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format as GLint,
                width,
                height,
                0,
                format,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const GLvoid,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }

        texture_id
    }
}

fn get_vec_from_parts(raw_data: *mut aiVector3D, size: c_uint) -> Vec<Vec3> {
    let slice = slice_from_raw_parts(raw_data, size as usize);
    if slice.is_null() {
        return vec![];
    }

    let raw_array = unsafe { slice.as_ref() }.unwrap();
    raw_array.iter().map(|aiv| vec3(aiv.x, aiv.y, aiv.z)).collect()
}
