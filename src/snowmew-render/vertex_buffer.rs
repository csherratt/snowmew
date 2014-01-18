use gl;
use gl::types::GLuint;

use std::mem;
use std::cast;
use std::ptr;

#[deriving(Clone, Default)]
pub struct VertexBuffer
{
    vertex_array: GLuint,
    vertex_buffer: GLuint,
    index_buffer: GLuint,

    vertex_buffer_len: uint,
    index_buffer_len: uint
}

impl VertexBuffer {
    pub fn new(vertex: &[f32], index: &[u32]) -> VertexBuffer
    {
        let mut vao = 0;
        let vbo: &mut[gl::types::GLuint] = &mut [0, 0];
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(2, vbo.unsafe_mut_ref(0));

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (vertex.len() * mem::size_of::<f32>())  as gl::types::GLsizeiptr,
                           cast::transmute(&vertex[0]),
                           gl::STATIC_DRAW
            );

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0,
                                    3,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    0,
                                    ptr::null()
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo[1]);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (index.len() * mem::size_of::<u32>())  as gl::types::GLsizeiptr,
                           cast::transmute(&index[0]),
                           gl::STATIC_DRAW
            );
        }

        /* todo check for errors */
        let error = gl::GetError();
        if error != 0 {
            println!("error {:x}", error);
        }

        VertexBuffer {
            vertex_array: vao,
            vertex_buffer: vbo[0],
            index_buffer: vbo[1],

            vertex_buffer_len: vertex.len(),
            index_buffer_len: index.len()
        }
    }

    pub fn bind(&self)
    {
        gl::BindVertexArray(self.vertex_array);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
    }
}

impl Drop for VertexBuffer
{
    fn drop(&mut self)
    {
        unsafe {
            //gl::DeleteVertexArrays(1, &self.vertex_array);
            //gl::DeleteBuffers(1, &self.vertex_buffer);
            //gl::DeleteBuffers(1, &self.index_buffer);
        }
    }
}