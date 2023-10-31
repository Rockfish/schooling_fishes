use crate::core::error::Error;
use glad_gl::gl;
use glad_gl::gl::{GLint, GLsizei, GLuint, GLvoid};
use image::ColorType;
use std::ffi::{c_uint, OsString};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

// pub struct Gamma(pub bool);
// pub struct FlipV(pub bool);

// utility function for loading a 2D texture from file

#[derive(Debug)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

#[derive(Debug, Copy, Clone)]
pub enum TextureType {
    None,
    Diffuse,
    Specular,
    Ambient,
    Emissive,
    Height,
    Normals,
    Shininess,
    Opacity,
    Displacement,
    Lightmap,
    Reflection,
    Unknown,
}

impl Display for TextureType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureType::Diffuse => write!(f, "texture_diffuse"),
            TextureType::Specular => write!(f, "texture_specular"),
            TextureType::Normals => write!(f, "texture_normal"),
            TextureType::Height => write!(f, "texture_height"),
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct TextureConfig {
    pub flip_v: bool,
    pub gamma_correction: bool,
    pub filter: TextureFilter,
    pub texture_type: TextureType,
}

#[derive(Debug)]
pub struct Texture {
    pub id: u32,
    pub texture_path: OsString,
    pub texture_type: TextureType,
}

impl Texture {
    pub fn new(texture_path: PathBuf, texture_config: &TextureConfig) -> Result<Texture, Error> {
        let id = Texture::load_texture(&texture_path, &texture_config)?;
        let texture = Texture {
            id,
            texture_path: texture_path.into_os_string(),
            texture_type: texture_config.texture_type.clone(),
        };
        Ok(texture)
    }

    pub fn load_texture(texture_path: &PathBuf, texture_config: &TextureConfig) -> Result<GLuint, Error> {
        let mut texture_id: GLuint = 0;

        let img = image::open(texture_path)?;
        let (width, height) = (img.width() as GLsizei, img.height() as GLsizei);

        let color_type = img.color();

        let img = if texture_config.flip_v { img.flipv() } else { img };

        unsafe {
            let mut internal_format: c_uint = 0;
            let mut data_format: c_uint = 0;
            match color_type {
                ColorType::L8 => {
                    internal_format = gl::RED;
                    data_format = gl::RED;
                }
                ColorType::Rgb8 => {
                    internal_format = if texture_config.gamma_correction { gl::SRGB } else { gl::RGB };
                    data_format = gl::RGB;
                }
                ColorType::Rgba8 => {
                    internal_format = if texture_config.gamma_correction { gl::SRGB_ALPHA } else { gl::RGBA };
                    data_format = gl::RGBA;
                }
                _ => panic!("no mapping for color type"),
            };

            let data = match color_type {
                ColorType::L8 => img.into_rgb8().into_raw(),
                ColorType::Rgb8 => img.into_rgb8().into_raw(),
                ColorType::Rgba8 => img.into_rgba8().into_raw(),
                _ => panic!("no mapping for color type"),
            };

            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as GLint,
                width,
                height,
                0,
                data_format,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const GLvoid,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            let param = if data_format == gl::RGBA { gl::CLAMP_TO_EDGE } else { gl::REPEAT };

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, param as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, param as GLint);

            match texture_config.filter {
                TextureFilter::Linear => {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                }
                TextureFilter::Nearest => {
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
                }
            }
        }
        Ok(texture_id)
    }
}
