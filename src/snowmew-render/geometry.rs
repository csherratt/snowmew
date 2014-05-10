use gl;

use std::mem;
use std::cast;
use std::ptr;
use std::vec;

use render::Context;

#[deriving(Clone, Default)]
pub struct Geometry {
    vertex_array: gl::types::GLuint,
    vertex_buffer: gl::types::GLuint,
    index_buffer: gl::types::GLuint,

    len: i32,
    draw_op: gl::types::GLenum,
    index_type: gl::types::GLenum,
}

pub trait GlType {
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

fn find_trig<IDX: GlType+Eq+Clone>(index: &[IDX], my_idx: uint, a: IDX, b: IDX) -> IDX {
    let my_idx = my_idx as int;
    for i in range(0, (index.len()/3) as int) {
        if i != my_idx {
            /* look for candidate */
            let mut found_a = -1;
            let mut found_b = -1;
            for j in range(0, 3) {
                if a == index[i*3+j] {
                    found_a = j;
                }
                if b == index[i*3+j] {
                    found_b = j;
                }
            }

            /* found a candidate */
            if found_a != -1 && found_b != -1  {
                for j in range(0, 3) {
                    if j != found_a && j != found_b {
                        return index[i*3+j].clone();
                    }
                }
            }
        }
    }
    fail!("Did not find vertex!");
}


pub fn to_triangles_adjacency<IDX: GlType+Eq+Clone>(index: &[IDX]) -> ~[IDX] {
    vec::build(Some(index.len() * 2), |emit| {
        for i in range(0, index.len()/3) {
            let a = &index[i*3];
            let b = &index[i*3+1];
            let c = &index[i*3+2];

            emit(a.clone());
            emit(find_trig(index, i, a.clone(), b.clone()).clone());
            emit(b.clone());
            emit(find_trig(index, i, b.clone(), c.clone()).clone());
            emit(c.clone());
            emit(find_trig(index, i, c.clone(), a.clone()).clone());
        }
    })
}

impl Geometry {
    pub fn triangles<VERT: GlType, IDX: GlType>(vertex: &[VERT], index: &[IDX]) -> Geometry {
        let mut vao = 0;
        let vbo: &mut[gl::types::GLuint] = &mut [0, 0];
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(2, vbo.unsafe_mut_ref(0));

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
        }

        /* todo check for errors */
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

    pub fn triangles_adjacency<VERT: GlType, IDX: GlType>(vertex: &[VERT], index: &[IDX]) -> Geometry {
        let mut vao = 0;
        let vbo: &mut[gl::types::GLuint] = &mut [0, 0];
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(2, vbo.unsafe_mut_ref(0));

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
        }

        /* todo check for errors */
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
            draw_op: gl::TRIANGLES_ADJACENCY
        }
    }

    pub fn lines<VERT: GlType, IDX: GlType+Clone>(vertex: &[VERT], index: &[IDX]) -> Geometry {
        let mut vao = 0;
        let vbo: &mut[gl::types::GLuint] = &mut [0, 0];

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(2, vbo.unsafe_mut_ref(0));

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
        }

        /* todo check for errors */
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
            draw_op: gl::LINES
        }
    }

    pub fn draw(&self, ctx: &mut Context) {
        ctx.vertex(self.vertex_array);
        ctx.element(self.index_buffer);
        let error = gl::GetError();
        if error != 0 {
            println(format!("error {:x}", error));
        }
        unsafe {
            gl::DrawElements(self.draw_op, self.len, self.index_type, ptr::null());
        }
        let error = gl::GetError();
        if error != 0 {
            println(format!("error {:x}", error));
        }

    }
}

impl Drop for Geometry {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteBuffers(1, &self.vertex_buffer);
            gl::DeleteBuffers(1, &self.index_buffer);
        }
    }
}