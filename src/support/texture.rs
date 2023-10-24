use glad_gl::gl;
use image::ColorType;
use std::ffi::c_uint;
use glad_gl::gl::{GLint, GLsizei, GLuint, GLvoid};

pub struct Gamma(pub bool);
pub struct FlipV(pub bool);

// utility function for loading a 2D texture from file
// ---------------------------------------------------
pub fn load_texture(path: &str, gammaCorrection: bool, flipv: bool) -> GLuint {
    let mut texture_id: GLuint = 0;

    let img = image::open(path).expect("Texture failed to load");
    let (width, height) = (img.width() as GLsizei, img.height() as GLsizei);

    let color_type = img.color();
    // let data = img.into_rgb8().into_raw();

    let img = if flipv { img.flipv() } else { img };

    unsafe {
        let mut internalFormat: c_uint = 0;
        let mut dataFormat: c_uint = 0;
        match color_type {
            ColorType::L8 => {
                internalFormat = gl::RED;
                dataFormat = gl::RED;
            }
            // ColorType::La8 => {}
            ColorType::Rgb8 => {
                internalFormat = if gammaCorrection { gl::SRGB } else { gl::RGB };
                dataFormat = gl::RGB;
            }
            ColorType::Rgba8 => {
                internalFormat = if gammaCorrection { gl::SRGB_ALPHA } else { gl::RGBA };
                dataFormat = gl::RGBA;
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

        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            internalFormat as GLint,
            width,
            height,
            0,
            dataFormat,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const GLvoid,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        // for this tutorial: use GL_CLAMP_TO_EDGE to prevent semi-transparent borders. Due to interpolation it takes texels from next repeat
        let param = if dataFormat == gl::RGBA { gl::CLAMP_TO_EDGE } else { gl::REPEAT };
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, param as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, param as GLint);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    }

    texture_id
}
