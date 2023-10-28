#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_variables)]

use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::os::raw::c_uint;

use russimp::scene::*;
use russimp::sys::*;
use russimp::*;

pub const aiTextureType_NONE: c_uint = 0x0;
pub const aiTextureType_DIFFUSE: c_uint = 0x1;
pub const aiTextureType_SPECULAR: c_uint = 0x2;
pub const aiTextureType_AMBIENT: c_uint = 0x3;
pub const aiTextureType_EMISSIVE: c_uint = 0x4;
pub const aiTextureType_HEIGHT: c_uint = 0x5;
pub const aiTextureType_NORMALS: c_uint = 0x6;
pub const aiTextureType_SHININESS: c_uint = 0x7;
pub const aiTextureType_OPACITY: c_uint = 0x8;
pub const aiTextureType_DISPLACEMENT: c_uint = 0x9;
pub const aiTextureType_LIGHTMAP: c_uint = 0xA;
pub const aiTextureType_REFLECTION: c_uint = 0xB;
pub const aiTextureType_UNKNOWN: c_uint = 0xC;

// This is just a lightweight wrapper around aiScene
#[derive(Debug)]
pub struct AiScene<'a> {
    pub assimp_scene: Option<&'a aiScene>,
}

impl AiScene<'_> {
    pub fn from_file(file_path: &str, flags: PostProcessSteps) -> Russult<AiScene> {
        let bitwise_flag = flags.into_iter().fold(0, |acc, x| acc | (x as u32));
        let file_path = CString::new(file_path).unwrap();

        let raw_scene = AiScene::get_scene_from_file(file_path, bitwise_flag);
        if raw_scene.is_none() {
            return Err(AiScene::get_error());
        }

        Ok(AiScene { assimp_scene: raw_scene })
    }

    #[inline]
    fn get_scene_from_file<'a>(string: CString, flags: u32) -> Option<&'a aiScene> {
        unsafe { aiImportFile(string.as_ptr(), flags).as_ref() }
    }

    fn get_error() -> RussimpError {
        let error_buf = unsafe { aiGetErrorString() };
        let error = unsafe { CStr::from_ptr(error_buf).to_string_lossy().into_owned() };
        RussimpError::Import(error)
    }

    #[inline]
    fn drop_scene(scene: Option<&aiScene>) {
        if let Some(content) = scene {
            unsafe {
                aiReleaseImport(content);
            }
        }
    }
}

impl Drop for AiScene<'_> {
    fn drop(&mut self) {
        AiScene::drop_scene(self.assimp_scene);
    }
}

pub fn get_material_texture_filename(material: *mut aiMaterial, texture_type: c_uint, index: u32) -> Result<String, String> {
    let mut path = MaybeUninit::uninit();
    let mut texture_mapping = MaybeUninit::uninit();
    let mut uv_index = MaybeUninit::uninit();
    let mut blend = MaybeUninit::uninit();
    let mut op = MaybeUninit::uninit();
    let mut map_mode: [u32; 2] = [0, 0];

    let mut flags = MaybeUninit::uninit();

    if unsafe {
        aiGetMaterialTexture(
            material,
            texture_type,
            index,
            path.as_mut_ptr(),
            texture_mapping.as_mut_ptr(),
            uv_index.as_mut_ptr(),
            blend.as_mut_ptr(),
            op.as_mut_ptr(),
            map_mode.as_mut_ptr() as *mut _,
            flags.as_mut_ptr(),
        )
    } == aiReturn_aiReturn_SUCCESS
    {
        let filename: String = unsafe { path.assume_init() }.into();
        return Ok(filename);
    }
    Err("Texture not found".to_string())
}
