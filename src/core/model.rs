use crate::core::assimp_scene::*;
use crate::core::error::Error;
use crate::core::error::Error::ModelError;
use crate::core::model_mesh::{ModelMesh, ModelVertex};
use crate::core::shader::Shader;
use crate::core::texture::{Texture, TextureConfig, TextureFilter, TextureType};
use glam::*;
use russimp::scene::*;
use russimp::sys::*;
use std::os::raw::c_uint;
use std::path::{Path, PathBuf};
use std::ptr::*;
use std::rc::Rc;

// model data
#[derive(Debug)]
pub struct Model {
    // stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    pub textures_loaded: Vec<Rc<Texture>>,
    pub meshes: Vec<ModelMesh>,
    pub directory: String,
    pub gammaCorrection: bool,
    pub flip_v: bool,
}

impl Model {
    pub fn new(path: &str, gamma: bool, flipv: bool) -> Result<Model, Error> {
        let mut model = Model {
            textures_loaded: vec![],
            meshes: vec![],
            directory: "".to_string(),
            gammaCorrection: gamma,
            flip_v: flipv,
        };
        model.load_model(path)?;
        Ok(model)
    }

    pub fn Draw(&self, shader: &Shader) {
        // draw each mesh in the model
        for mesh in &self.meshes {
            mesh.Draw(shader);
        }
    }

    // loads a model with supported ASSIMP extensions from file and stores the resulting meshes in the meshes vector.
    fn load_model(&mut self, path: &str) -> Result<(), Error> {
        let scene = AssimpScene::from_file(
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

                if let Some(assimp_scene) = scene.assimp_scene {
                    self.process_node(assimp_scene.mRootNode, assimp_scene)?;
                }
                Ok(())
            }
            Err(err) => Err(ModelError(err)),
        }
    }

    fn process_node(&mut self, node: *mut aiNode, scene: &aiScene) -> Result<(), Error> {
        // process each mesh located at the current node
        // println!("{:?}", unsafe { (*node).mName });

        let slice = slice_from_raw_parts(scene.mMeshes, scene.mNumMeshes as usize);
        let assimp_meshes = unsafe { slice.as_ref() }.unwrap();

        for i in 0..assimp_meshes.len() {
            let mesh = self.process_mesh(assimp_meshes[i], scene);
            self.meshes.push(mesh?);
        }

        // Process children nodes
        let slice = unsafe { slice_from_raw_parts((*node).mChildren, (*node).mNumChildren as usize) };

        if let Some(child_nodes) = unsafe { slice.as_ref() } {
            for i in 0..child_nodes.len() {
                self.process_node(child_nodes[i], scene)?;
            }
        }
        Ok(())
    }

    fn process_mesh(&mut self, scene_mesh: *mut aiMesh, scene: &aiScene) -> Result<ModelMesh, Error> {
        let scene_mesh = unsafe { *scene_mesh };

        let mut vertices: Vec<ModelVertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        let mut textures: Vec<Rc<Texture>> = vec![];

        let assimp_vertices = get_vec_from_parts(scene_mesh.mVertices, scene_mesh.mNumVertices);
        let assimp_normals = get_vec_from_parts(scene_mesh.mNormals, scene_mesh.mNumVertices);
        let assimp_tangents = get_vec_from_parts(scene_mesh.mTangents, scene_mesh.mNumVertices);
        let assimp_bitangents = get_vec_from_parts(scene_mesh.mBitangents, scene_mesh.mNumVertices);

        // a vertex can contain up to 8 different texture coordinates. We thus make the assumption that we won't
        // use models where a vertex can have multiple texture coordinates so we always take the first set (0).
        let texture_coords = if !scene_mesh.mTextureCoords.is_empty() {
            get_vec_from_parts(scene_mesh.mTextureCoords[0], assimp_vertices.len() as u32)
        } else {
            vec![]
        };

        for i in 0..assimp_vertices.len() {
            let mut vertex = ModelVertex::new();

            // positions
            vertex.position = assimp_vertices[i]; // Vec3 has Copy trait

            // normals
            if !assimp_normals.is_empty() {
                vertex.normal = assimp_normals[i];
            }

            // texture coordinates
            if !texture_coords.is_empty() {
                // texture coordinates
                vertex.tex_coords = vec2(texture_coords[i].x, texture_coords[i].y);
                // tangent
                vertex.tangent = assimp_tangents[i];
                // bitangent
                vertex.bi_tangent = assimp_bitangents[i];
            } else {
                vertex.tex_coords = vec2(0.0, 0.0);
            }
            vertices.push(vertex);
        }
        // now walk through each of the mesh's faces (a face is a mesh its triangle) and retrieve the corresponding vertex indices.
        let assimp_faces = unsafe { slice_from_raw_parts(scene_mesh.mFaces, scene_mesh.mNumFaces as usize).as_ref() }.unwrap();

        for i in 0..assimp_faces.len() {
            let face = assimp_faces[i];
            let assimp_indices = unsafe { slice_from_raw_parts(face.mIndices, face.mNumIndices as usize).as_ref() }.unwrap();
            indices.extend(assimp_indices.iter());
        }

        // process materials
        let assimp_materials = unsafe { slice_from_raw_parts((*scene).mMaterials, (*scene).mNumMaterials as usize).as_ref() }.unwrap();
        let material_index = scene_mesh.mMaterialIndex as usize;
        let assimp_material = assimp_materials[material_index];

        // we assume a convention for sampler names in the shaders. Each diffuse texture should be named
        // as 'texture_diffuseN' where N is a sequential number ranging from 1 to MAX_SAMPLER_NUMBER.
        // Same applies to other texture as the following list summarizes:
        // diffuse: texture_diffuseN
        // specular: texture_specularN
        // normal: texture_normalN

        // 1. diffuse maps
        let diffuseMaps = self.load_material_textures(assimp_material, TextureType::Diffuse)?;
        textures.extend(diffuseMaps);
        // 2. specular maps
        let specularMaps = self.load_material_textures(assimp_material, TextureType::Specular)?;
        textures.extend(specularMaps);
        // 3. normal maps
        let normalMaps = self.load_material_textures(assimp_material, TextureType::Height)?;
        textures.extend(normalMaps);
        // 4. height maps
        let heightMaps = self.load_material_textures(assimp_material, TextureType::Ambient)?;
        textures.extend(heightMaps);

        let mesh = ModelMesh::new(vertices, indices, textures);
        Ok(mesh)
    }

    fn load_material_textures(&mut self, assimp_material: *mut aiMaterial, texture_type: TextureType) -> Result<Vec<Rc<Texture>>, Error> {
        let mut textures: Vec<Rc<Texture>> = vec![];

        let texture_count = unsafe { aiGetMaterialTextureCount(assimp_material, texture_type.into()) };

        for i in 0..texture_count {
            let texture_filename = get_material_texture_filename(assimp_material, texture_type, i as u32)?;
            let full_path = PathBuf::from(&self.directory).join(&texture_filename);

            let loaded_texture = self
                .textures_loaded
                .iter()
                .find(|t| t.texture_path == full_path.clone().into_os_string());

            match loaded_texture {
                None => {
                    let texture = Rc::new(Texture::new(
                        full_path,
                        &TextureConfig {
                            flip_v: self.flip_v,
                            gamma_correction: self.gammaCorrection,
                            filter: TextureFilter::Linear,
                            texture_type,
                        },
                    )?);
                    self.textures_loaded.push(texture.clone());
                    textures.push(texture.clone());
                }
                Some(texture) => textures.push(texture.clone()),
            }
        }
        Ok(textures)
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
