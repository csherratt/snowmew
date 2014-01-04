use gl;

use shader::Shader;

pub struct Context {
    viewport: ((i32, i32), (i32, i32)),
    program: u32,
    shader: u32,
    blend: (u32, u32),
    elements: u32,
    vertex: u32
}

impl Context
{
    pub fn new() -> Context
    {
        let viewport = &mut [0i32, 0i32, 0i32, 0i32];
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, viewport.unsafe_mut_ref(0));
        }

        Context {
            viewport: ((viewport[0], viewport[1]), (viewport[2], viewport[3])),
            program: 0,
            shader: 0,
            blend: (0, 0),
            elements: 0,
            vertex: 0
        }
    }

    pub fn viewport(&mut self, pos: (i32, i32), size: (i32, i32))
    {
        let (x, y) = pos;
        let (w, h) = size;
        let ((ox, oy), (ow, oh)) = self.viewport;
        if ox != x || oy != y || ow != w || oh != h {
            gl::Viewport(x, y, w, h);
            gl::Scissor(x, y, w, h);
        }
        self.viewport = ((x, y), (w, h));
    }

    pub fn get_viewport(&self) -> ((i32, i32), (i32, i32))
    {
        self.viewport
    }

    pub fn program(&mut self, program: u32)
    {
        if program == self.program {
            gl::UseProgram(program);
        }
        self.program = program;
    }

    pub fn get_program(&self) -> u32
    {
        self.program
    }

    pub fn shader(&mut self, shader: &Shader)
    {
        if shader.program != self.program {
            gl::UseProgram(shader.program);
            self.program = shader.program;
        }
        if shader.blend != self.blend {
            let (s, d) = shader.blend;
            gl::BlendFunc(s, d);
            self.blend = shader.blend;
        }
    }

    pub fn vertex(&mut self, vertex: u32)
    {
        if self.vertex != vertex {
            gl::BindVertexArray(vertex);
            self.vertex = vertex;
        }
    }

    pub fn element(&mut self, element: u32)
    {
        if self.elements != element {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element);
            self.elements = element;
        }
    }   

}