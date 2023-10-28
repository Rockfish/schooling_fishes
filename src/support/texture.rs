use crate::support::error::Error;
use glad_gl::gl;
use glad_gl::gl::{GLint, GLsizei, GLuint, GLvoid};
use image::ColorType;
use std::ffi::c_uint;
use std::path::Path;

// pub struct Gamma(pub bool);
// pub struct FlipV(pub bool);

// utility function for loading a 2D texture from file

pub fn load_texture_from_str(path: &str, gama_correction: bool, flipv: bool) -> Result<GLuint, Error> {
    load_texture(Path::new(path), gama_correction, flipv)
}

pub fn load_texture(path: &Path, gama_correction: bool, flipv: bool) -> Result<GLuint, Error> {
    let mut texture_id: GLuint = 0;

    let img = image::open(path)?;
    let (width, height) = (img.width() as GLsizei, img.height() as GLsizei);

    let color_type = img.color();
    // let data = img.into_rgb8().into_raw();

    let img = if flipv { img.flipv() } else { img };

    unsafe {
        let mut internal_format: c_uint = 0;
        let mut data_format: c_uint = 0;
        match color_type {
            ColorType::L8 => {
                internal_format = gl::RED;
                data_format = gl::RED;
            }
            // ColorType::La8 => {}
            ColorType::Rgb8 => {
                internal_format = if gama_correction { gl::SRGB } else { gl::RGB };
                data_format = gl::RGB;
            }
            ColorType::Rgba8 => {
                internal_format = if gama_correction { gl::SRGB_ALPHA } else { gl::RGBA };
                data_format = gl::RGBA;
            }
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

        // println!("texture format: {:#02x}",dataFormat);

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

        // for this tutorial: use GL_CLAMP_TO_EDGE to prevent semi-transparent borders. Due to interpolation it takes texels from next repeat
        let param = if data_format == gl::RGBA { gl::CLAMP_TO_EDGE } else { gl::REPEAT };
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, param as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, param as GLint);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    }

    Ok(texture_id)
}
