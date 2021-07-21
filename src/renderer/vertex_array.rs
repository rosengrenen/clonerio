use super::vertex_buffer::VertexBuffer;

#[derive(Debug)]
pub struct VertexArray {
    id: u32,
    elements: u32,
}

impl VertexArray {
    pub unsafe fn new(buffers: &[VertexBuffer]) -> Self {
        let mut id = 0;
        gl::CreateVertexArrays(1, &mut id);

        let mut attrib_index = 0;
        for (binding_index, buffer) in buffers.iter().enumerate() {
            let stride = buffer
                .layout
                .iter()
                .fold(0, |prev, element| prev + element.mem_size());
            gl::VertexArrayVertexBuffer(id, binding_index as u32, buffer.id, 0, stride as i32);

            let mut offset = 0;
            for element in buffer.layout.iter() {
                gl::EnableVertexArrayAttrib(id, attrib_index as u32);
                gl::VertexArrayAttribFormat(
                    id,
                    attrib_index as u32,
                    element.size as i32,
                    element.to_raw_enum(),
                    gl::FALSE,
                    offset,
                );
                gl::VertexArrayAttribBinding(id, attrib_index as u32, binding_index as u32);

                offset += element.mem_size();
                attrib_index += 1;
            }
        }

        let elements = buffers.iter().min_by_key(|b| b.elements).unwrap().elements;

        Self { id, elements }
    }

    pub unsafe fn bind(&self) {
        gl::BindVertexArray(self.id);
    }
}
