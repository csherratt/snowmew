use gl;

use std::mem;
use std::cast;
use std::ptr;

pub struct Geometry {
    vertex_array: gl::types::GLuint,
    vertex_buffer: gl::types::GLuint,
    index_buffer: gl::types::GLuint,

    len: i32,
    draw_op: gl::types::GLenum,
    index_type: gl::types::GLenum,
}

trait GlType {
    fn gl_type(&self) -> gl::types::GLenum;
}

impl GlType for u32 {
    fn gl_type(&self) -> gl::types::GLenum { gl::UNSIGNED_INT }
} 

impl GlType for u16 {
    fn gl_type(&self) -> gl::types::GLenum { gl::UNSIGNED_SHORT }
} 

impl GlType for u8 {
    fn gl_type(&self) -> gl::types::GLenum { gl::UNSIGNED_BYTE }
} 

impl GlType for f32 {
    fn gl_type(&self) -> gl::types::GLenum { gl::FLOAT }
} 

impl Geometry {
    pub fn triangles<VERT: GlType, IDX: GlType>(vertex: &[VERT], index: &[IDX]) -> Geometry
    {
        let mut vao = 0;
        let vbo: &mut[gl::types::GLuint] = &mut [0, 0];
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            do vbo.as_mut_buf |ptr, _| { 
                gl::GenBuffers(2, ptr);
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (vertex.len() * mem::size_of::<VERT>())  as gl::types::GLsizeiptr,
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
                           (index.len() * mem::size_of::<IDX>())  as gl::types::GLsizeiptr,
                           cast::transmute(&index[0]),
                           gl::STATIC_DRAW
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }

        /* todo check for erros */
        let error = gl::GetError();
        if error != 0 {
            println(format!("error {:x}", error));
        }

        Geometry {
            vertex_array: vao,
            vertex_buffer: vbo[0],
            index_buffer: vbo[1],
            len: index.len() as i32,
            index_type: index[0].gl_type(),
            draw_op: gl::TRIANGLES
        }
    }

    pub fn draw(&self) {
        gl::BindVertexArray(self.vertex_array);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
        let error = gl::GetError();
        if error != 0 {
            println(format!("error {:x}", error));
        }
        unsafe {
            gl::DrawElements(self.draw_op, self.len, self.index_type, ptr::null());
            //gl::DrawArrays(self.draw_op, 0, 3);
        }
        let error = gl::GetError();
        if error != 0 {
            println(format!("error {:x}", error));
        }
        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    }
}
