#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_assignments)]

use glad_gl::gl;
use glad_gl::gl::{GLchar, GLint, GLuint};
use glam::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::Path;
use std::ptr;

use crate::*;

pub struct Shader {
    pub id: ShaderId,
}

impl Shader {
    pub fn new(vertexPath: &str, fragmentPath: &str, geometryPath: Option<&str>) -> Result<Self, String> {
        let mut shader = Shader { id: 0 };
        let mut vertexCode: String = Default::default();
        let mut fragmentCode: String = Default::default();
        let mut geometryCode: String = Default::default();

        match read_file(vertexPath) {
            Ok(content) => vertexCode = content,
            Err(error) => return Err(error.to_string()),
        }

        match read_file(fragmentPath) {
            Ok(content) => fragmentCode = content,
            Err(error) => return Err(error.to_string()),
        }

        if let Some(geometryPath) = geometryPath {
            match read_file(geometryPath) {
                Ok(content) => geometryCode = content,
                Err(error) => return Err(error.to_string()),
            }
        }

        unsafe {
            // vertex shader
            let vertexShader = gl::CreateShader(gl::VERTEX_SHADER);
            let c_string = c_string!(vertexCode);
            gl::ShaderSource(vertexShader, 1, &c_string.as_ptr(), ptr::null());
            gl::CompileShader(vertexShader);

            if let Err(error) = checkCompileErrors(vertexShader, "VERTEX") {
                return Err(error);
            }

            // fragment shader
            let fragmentShader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_string = c_string!(fragmentCode);
            gl::ShaderSource(fragmentShader, 1, &c_string.as_ptr(), ptr::null());
            gl::CompileShader(fragmentShader);

            if let Err(error) = checkCompileErrors(fragmentShader, "FRAGMENT") {
                return Err(error);
            }

            // geometry shader
            let mut geometryShader: GLuint = 0;
            if let Some(_geometryPath) = geometryPath {
                geometryShader = gl::CreateShader(gl::GEOMETRY_SHADER);
                let c_string = c_string!(geometryCode);
                gl::ShaderSource(geometryShader, 1, &c_string.as_ptr(), ptr::null());
                gl::CompileShader(geometryShader);

                if let Err(error) = checkCompileErrors(geometryShader, "GEOMETRY") {
                    return Err(error);
                }
            }

            // shader program
            shader.id = gl::CreateProgram();
            // link the first program object
            gl::AttachShader(shader.id, vertexShader);
            gl::AttachShader(shader.id, fragmentShader);
            if let Some(_geometryPath) = geometryPath {
                gl::AttachShader(shader.id, geometryShader);
            }
            gl::LinkProgram(shader.id);

            if let Err(error) = checkCompileErrors(shader.id, "PROGRAM") {
                return Err(error);
            }

            // delete the shaders as they're linked into our program now and no longer necessary
            gl::DeleteShader(vertexShader);
            gl::DeleteShader(fragmentShader);
            if let Some(_geometryPath) = geometryPath {
                gl::DeleteShader(geometryShader);
            }
        }

        Ok(shader)
    }

    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    // utility uniform functions
    // ------------------------------------------------------------------------
    pub fn setBool(&self, name: &str, value: bool) {
        unsafe {
            let v = if value { 1 } else { 0 };
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform1i(location, v);
        }
    }

    // ------------------------------------------------------------------------
    pub fn setInt(&self, name: &str, value: i32) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform1i(location, value);
        }
    }

    // ------------------------------------------------------------------------
    pub fn setFloat(&self, name: &str, value: f32) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform1f(location, value);
        }
    }

    // ------------------------------------------------------------------------
    pub fn setVec2(&self, name: &str, value: &Vec2) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform2fv(location, 1, value.to_array().as_ptr());
        }
    }

    // ------------------------------------------------------------------------
    pub fn setVec2_xy(&self, name: &str, x: f32, y: f32) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform2f(location, x, y);
        }
    }

    // ------------------------------------------------------------------------
    pub fn setVec3(&self, name: &str, value: &Vec3) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform3fv(location, 1, value.to_array().as_ptr());
        }
    }

    // ------------------------------------------------------------------------
    pub fn setVec3_xyz(&self, name: &str, x: f32, y: f32, z: f32) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform3f(location, x, y, z);
        }
    }

    // ------------------------------------------------------------------------
    pub fn setVec4(&self, name: &str, value: &Vec4) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform4fv(location, 1, value.to_array().as_ptr());
        }
    }

    // ------------------------------------------------------------------------
    pub fn setVec4_xyzw(&self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::Uniform4f(location, x, y, z, w);
        }
    }

    // ------------------------------------------------------------------------
    pub fn setMat2(&self, name: &str, mat: &Mat2) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::UniformMatrix2fv(location, 1, gl::FALSE, mat.to_cols_array().as_ptr());
        }
    }

    // ------------------------------------------------------------------------
    pub fn setMat3(&self, name: &str, mat: &Mat3) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::UniformMatrix3fv(location, 1, gl::FALSE, mat.to_cols_array().as_ptr());
        }
    }

    // ------------------------------------------------------------------------
    pub fn setMat4(&self, name: &str, matrix: &Mat4) {
        unsafe {
            let c_string = c_string!(name);
            let location = gl::GetUniformLocation(self.id, c_string.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.to_cols_array().as_ptr());
        }
    }
}

fn read_file(filename: &str) -> Result<String, Error> {
    let mut content: String = Default::default();
    let mut file = File::open(Path::new(filename))?;
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn checkCompileErrors(shaderId: u32, checkType: &str) -> Result<(), String> {
    unsafe {
        let mut status = gl::FALSE as GLint;

        if checkType != "PROGRAM" {
            gl::GetShaderiv(shaderId, gl::COMPILE_STATUS, &mut status);
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shaderId, gl::INFO_LOG_LENGTH, &mut len);
                // Subtract 1 to skip the trailing null character.
                let mut infoLog = vec![0; len as usize - 1];
                gl::GetProgramInfoLog(shaderId, 1024, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
                return Err(String::from_utf8_lossy(&infoLog).to_string());
            }
        } else {
            gl::GetProgramiv(shaderId, gl::LINK_STATUS, &mut status);
            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetProgramiv(shaderId, gl::INFO_LOG_LENGTH, &mut len);
                // Subtract 1 to skip the trailing null character.
                let mut infoLog = vec![0; len as usize - 1];
                gl::GetProgramInfoLog(shaderId, 1024, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
                let error_msg = String::from_utf8_lossy(&infoLog).to_string();
                return Err(error_msg);
            }
        }
    }
    Ok(())
}
