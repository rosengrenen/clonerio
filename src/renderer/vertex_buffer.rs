use std::mem;

use gl::types::{GLfloat, GLsizeiptr, GLuint};

use super::VertexBufferElement;

#[derive(Debug)]
pub struct VertexBuffer {
    pub id: GLuint,
    pub elements: u32,
    pub layout: Vec<VertexBufferElement>,
}

impl VertexBuffer {
    pub unsafe fn new(data: &[GLfloat], layout: Vec<VertexBufferElement>) -> Self {
        let mut id = 0;
        gl::CreateBuffers(1, &mut id);
        gl::NamedBufferStorage(
            id,
            (data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&data[0]),
            gl::DYNAMIC_STORAGE_BIT,
        );

        let elements =
            data.len() as u32 / layout.iter().fold(0, |prev, element| prev + element.size);
        Self {
            id,
            elements,
            layout,
        }
    }
}
