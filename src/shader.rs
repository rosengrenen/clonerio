use cgmath::{Matrix, Matrix2, Matrix4, Vector3, Vector4};
use gl::types::*;
use std::{
    ffi::{CStr, CString},
    fs, ptr, str,
};

pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn from_file(vs_path: &str, fs_path: &str) -> Self {
        let vs_source = fs::read_to_string(format!("assets/shaders/{}", vs_path))
            .expect(&format!("Could not read vertex shader {}", vs_path));
        let fs_source = fs::read_to_string(format!("assets/shaders/{}", fs_path))
            .expect(&format!("Could not read fragment shader {}", fs_path));

        Self::from_source(&vs_source, &fs_source)
    }

    pub fn from_source(vs_source: &str, fs_source: &str) -> Self {
        let vs_id = Self::compile_shader(vs_source, gl::VERTEX_SHADER);
        let fs_id = Self::compile_shader(fs_source, gl::FRAGMENT_SHADER);
        let program_id = Self::link_program(vs_id, fs_id);
        unsafe {
            gl::DeleteShader(vs_id);
            gl::DeleteShader(fs_id);
        }

        Self { id: program_id }
    }

    pub fn enable(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn set_mat2(&self, name: &CStr, matrix: Matrix2<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, name.as_ptr());
            gl::UniformMatrix2fv(location, 1, gl::FALSE, matrix.as_ptr());
        }
    }

    pub fn set_mat4(&self, name: &CStr, matrix: Matrix4<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, name.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_ptr());
        }
    }

    pub fn set_vec3(&self, name: &CStr, vec: Vector3<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, name.as_ptr());
            gl::Uniform3f(location, vec.x, vec.y, vec.z);
        }
    }

    pub fn set_vec4(&self, name: &CStr, vec: Vector4<f32>) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, name.as_ptr());
            gl::Uniform4f(location, vec.x, vec.y, vec.z, vec.w);
        }
    }

    pub fn set_int(&self, name: &CStr, int: i32) {
        unsafe {
            let location = gl::GetUniformLocation(self.id, name.as_ptr());
            gl::Uniform1i(location, int);
        }
    }

    fn compile_shader(src: &str, ty: GLenum) -> GLuint {
        let shader;
        unsafe {
            shader = gl::CreateShader(ty);

            let c_str = CString::new(src.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "{}",
                    str::from_utf8(&buf)
                        .ok()
                        .expect("ShaderInfoLog not valid utf8")
                );
            }
        }
        shader
    }

    fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
            // Get the link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "{}",
                    str::from_utf8(&buf)
                        .ok()
                        .expect("ProgramInfoLog not valid utf8")
                );
            }
            program
        }
    }
}
