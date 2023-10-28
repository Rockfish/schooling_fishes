use crate::support::ai_scene::*;
use crate::support::error::Error;
use crate::support::error::Error::ModelError;
use crate::support::model_mesh::{ModelMesh, ModelTexture, ModelVertex};
use crate::support::texture::load_texture;
use crate::support::ShaderId;
use glam::*;
use russimp::scene::*;
use russimp::sys::*;
use std::os::raw::c_uint;
use std::path::{Path, PathBuf};
use std::ptr::*;

// model data
#[derive(Debug)]
pub struct Model {
    // stores all the textures loaded so far, optimization to make sure textures aren't loaded more than once.
    pub textures_loaded: Vec<ModelTexture>,
    pub meshes: Vec<ModelMesh>,
    pub directory: String,
    pub gammaCorrection: bool,
    pub flipv: bool,
}

pub struct Gamma(pub bool);
pub struct FlipV(pub bool);

impl Model {
    pub fn new(path: &str, gamma: Gamma, flipv: FlipV) -> Result<Model, Error> {
        let mut model = Model {
            textures_loaded: vec![],
            meshes: vec![],
            directory: "".to_string(),
            gammaCorrection: gamma.0,
            flipv: flipv.0,
        };
        model.load_model(path)?;
        Ok(model)
    }

    pub fn Draw(&self, shader_id: ShaderId) {
        for mesh in &self.meshes {
            mesh.Draw(shader_id);
        }
    }

    // loads a model with supported ASSIMP extensions from file and stores the resulting meshes in the meshes vector.
    fn load_model(&mut self, path: &str) -> Result<(), Error> {
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

                if let Some(ai_scene) = scene.assimp_scene {
                    self.process_node(ai_scene.mRootNode, ai_scene)?;
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
        let ai_meshes = unsafe { slice.as_ref() }.unwrap();

        for i in 0..ai_meshes.len() {
            let mesh = self.process_mesh(ai_meshes[i], scene);
            self.meshes.push(mesh?);
        }

        // Process children nodes
        let slice = unsafe { slice_from_raw_parts((*node).mChildren, (*node).mNumChildren as usize) };

        if let Some(child_nodes) = unsafe { slice.as_ref() } {
            for i in 0..child_nodes.len() {
                // println!("{:#?}", unsafe { (*child_nodes[i]).mName });
                self.process_node(child_nodes[i], scene)?;
            }
        }
        Ok(())
    }

    fn process_mesh(&mut self, scene_mesh: *mut aiMesh, scene: &aiScene) -> Result<ModelMesh, Error> {
        let scene_mesh = unsafe { *scene_mesh };

        let mut vertices: Vec<ModelVertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        let mut textures: Vec<ModelTexture> = vec![];

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
            let mut vertex = ModelVertex::new();

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
        let diffuseMaps = self.load_material_textures(ai_material, aiTextureType_DIFFUSE, "texture_diffuse")?;
        textures.extend(diffuseMaps);
        // 2. specular maps
        let specularMaps = self.load_material_textures(ai_material, aiTextureType_SPECULAR, "texture_specular")?;
        textures.extend(specularMaps);
        // 3. normal maps
        let normalMaps = self.load_material_textures(ai_material, aiTextureType_HEIGHT, "texture_normal")?;
        textures.extend(normalMaps);
        // 4. height maps
        let heightMaps = self.load_material_textures(ai_material, aiTextureType_AMBIENT, "texture_height")?;
        textures.extend(heightMaps);

        let mesh = ModelMesh::new(vertices, indices, textures);
        Ok(mesh)
    }

    fn load_material_textures(&mut self, ai_material: *mut aiMaterial, ai_texture_type: c_uint, typeName: &str) -> Result<Vec<ModelTexture>, Error> {
        let mut textures: Vec<ModelTexture> = vec![];

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

                    let id = load_texture(&filepath, false, self.flipv)?;

                    let texture = ModelTexture {
                        id,
                        texture_type: typeName.to_string(),
                        path: filename,
                    };
                    textures.push(texture.clone());
                    self.textures_loaded.push(texture);
                }
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
