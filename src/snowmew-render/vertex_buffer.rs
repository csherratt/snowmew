use gl;
use gl::types::GLuint;

use std::mem;
use std::cast;
use std::ptr;

use libc::c_void;

use snowmew::geometry::{Vertex, VertexGeo, VertexGeoTex, VertexGetTexNorm};
use snowmew::geometry::{Geo, GeoTex, GeoTexNorm, Empty};

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
    pub fn new(vertex: &Vertex, index: &[u32]) -> VertexBuffer
    {
        let mut vao = 0;
        let vbo: &mut[gl::types::GLuint] = &mut [0, 0];

        let (vertex_size, index_size) = unsafe {
            let (addr, size, stride) = match *vertex {
                Geo(ref data) => {
                    (cast::transmute(data.get(0)),
                     data.len() * mem::size_of::<VertexGeo>(),
                     mem::size_of::<VertexGeo>())
                },
                GeoTex(ref data) => {
                    (cast::transmute(data.get(0)),
                     data.len() * mem::size_of::<VertexGeoTex>(),
                     mem::size_of::<VertexGeoTex>())
                },
                GeoTexNorm(ref data) => {
                    (cast::transmute(data.get(0)),
                     data.len() * mem::size_of::<VertexGetTexNorm>(),
                     mem::size_of::<VertexGetTexNorm>())
                },
                Empty => fail!("Should not be empty"),
            };
            let stride = stride as i32;

            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(2, vbo.unsafe_mut_ref(0));

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo[0]);
            gl::BufferData(gl::ARRAY_BUFFER,
                           size as gl::types::GLsizeiptr,
                           addr,
                           gl::STATIC_DRAW
            );

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());

            match *vertex {
                Empty | Geo(_) => (),
                GeoTex(_) | GeoTexNorm(_) => {
                    gl::EnableVertexAttribArray(1);
                    gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, 12 as *c_void);
                }
            }

            match *vertex {
                Empty | Geo(_) | GeoTex(_) => (),
                GeoTexNorm(_) => {
                    gl::EnableVertexAttribArray(2);
                    gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, stride, 20 as *c_void);
                }
            }

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo[1]);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                           (index.len() * mem::size_of::<u32>())  as gl::types::GLsizeiptr,
                           cast::transmute(&index[0]),
                           gl::STATIC_DRAW
            );

            (size, index.len())
        };

        /* todo check for errors */
        let error = gl::GetError();
        if error != 0 {
            fail!("error {:x}", error);
        }

        VertexBuffer {
            vertex_array: vao,
            vertex_buffer: vbo[0],
            index_buffer: vbo[1],

            vertex_buffer_len: vertex_size,
            index_buffer_len: index_size
        }
    }

    pub fn bind(&self)
    {
        gl::BindVertexArray(self.vertex_array);
        //gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
        //gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
    }
}

//impl Drop for VertexBuffer
//{
    //fn drop(&mut self)
    //{
        //unsafe {
            //gl::DeleteVertexArrays(1, &self.vertex_array);
            //gl::DeleteBuffers(1, &self.vertex_buffer);
            //gl::DeleteBuffers(1, &self.index_buffer);
        //}
   //}
//}