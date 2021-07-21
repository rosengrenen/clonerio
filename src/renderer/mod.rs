pub mod debug;
pub mod shader;
pub mod texture;
pub mod vertex_array;
pub mod vertex_buffer;

use std::mem;

use gl::types::GLfloat;

#[derive(Debug)]
pub enum GlType {
    Float,
}

impl GlType {
    pub fn to_raw_enum(&self) -> u32 {
        match *self {
            GlType::Float => gl::FLOAT,
        }
    }

    pub fn mem_size(&self) -> u32 {
        match *self {
            GlType::Float => mem::size_of::<GLfloat>() as u32,
        }
    }
}

#[derive(Debug)]
pub struct VertexBufferElement {
    size: u32,
    type_: GlType,
}

impl VertexBufferElement {
    pub fn floats(size: u32) -> Self {
        Self {
            size,
            type_: GlType::Float,
        }
    }

    pub fn to_raw_enum(&self) -> u32 {
        self.type_.to_raw_enum()
    }

    pub fn mem_size(&self) -> u32 {
        self.size * self.type_.mem_size()
    }
}
