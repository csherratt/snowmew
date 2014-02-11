use sync::MutexArc;
use glfw::{Window, Windowed, FullScreen, Monitor, WindowMode};
use input::{InputManager, InputHandle};

use glfw;
use gl;

use ovr;

pub struct Display
{
    priv window: MutexArc<Window>,
    priv handle: InputHandle,
    priv hmd_info: Option<ovr::HMDInfo>
}

pub struct RenderContext
{
    priv context: Window
}

impl Display
{
    fn window(im: &mut InputManager, size: (u32, u32), win: WindowMode) -> Option<(Display, InputHandle)>
    {
        let (width, height) = size;
        let window = Window::create(width, height, "Snoawmew", win);
        let window = match window {
            Some(window) => window,
            None => return None
        };

        println!("{:?}", window.get_context_version());

        window.make_context_current();
        gl::load_with(glfw::get_proc_address);

        window.show();
        let window = MutexArc::new(window);
        let handle = im.add_window(window.clone());

        Some((Display {
            window: window,
            handle: handle.clone(),
            hmd_info: None
        },
        handle))       
    }

    pub fn new_window(im: &mut InputManager, size: (uint, uint)) -> Option<(Display, InputHandle)>
    {
        let (w, h) = size;
        let size = (w as u32, h as u32);
        Display::window(im, size, Windowed)
    }

    pub fn new_primary(im: &mut InputManager) -> Option<(Display, InputHandle)>
    {
        let primary = Monitor::get_primary().unwrap();
        let mode = primary.get_video_mode().unwrap();

        let size = (mode.width as u32, mode.height as u32);
        Display::window(im, size, FullScreen(primary))
    }

    pub fn new_ovr(im: &mut InputManager) -> Option<(Display, InputHandle)>
    {
        if !im.setup_ovr() {
            return None;
        }
        
        let info = {
            let mgr = match im.ovr_manager() {
                Some(mgr) => mgr,
                None => return None
            };
            let hmd = match mgr.enumerate() {
                Some(hmd) => hmd,
                None => return None
            };

            hmd.get_info()
        };

        let (width, height) = info.resolution();
        let (width, height) = (width as u32, height as u32);

        let mut idx = None;
        let monitors = Monitor::get_connected();   
        for (i, m) in monitors.iter().enumerate() {
            match m.get_video_mode() {
                Some(mode) => {
                    if mode.width == width && mode.height == height {
                        idx = Some(i);
                        break;
                    }
                },
                None => ()
            }
        }

        let idx = match idx {
            Some(i) => i,
            None => return None
        };

        let win = Display::window(im, (width, height), FullScreen(monitors[idx]));
        match win {
            Some((mut win, input_handle)) => {
                win.hmd_info = Some(info);
                Some((win, input_handle))
            },
            None => None
        }
    }

    pub fn size(&self) -> (uint, uint)
    {
        unsafe {
            self.window.unsafe_access(|window| {
                let (w, h) = window.get_size();
                (w as uint, h as uint)
            })
        }
    }

    pub fn swap_buffers(&self)
    {
        unsafe {
            self.window.unsafe_access(|window| {
                window.swap_buffers();
            })
        }
    }

    pub fn is_hmd(&self) -> bool
    {
        self.hmd_info.is_some()
    }

    pub fn hmd(&self) -> ovr::HMDInfo
    {
        self.hmd_info.unwrap().clone()
    }

    pub fn set_cursor_mode(&mut self, cm: glfw::CursorMode)
    {
        unsafe {
            self.window.unsafe_access(|win| {
                win.set_cursor_mode(cm);
            });
        }
    }

    pub fn make_current(&mut self)
    {
        unsafe {
            self.window.unsafe_access(|win| {
                win.make_context_current();
            });
        }
    }

    pub fn make_render_context(&mut self) -> Option<RenderContext>
    {
        unsafe {
            self.window.unsafe_access(|win| {
                let window = win.create_shared(0, 0, "Render Context", glfw::Windowed);
                match window {
                    Some(win) => {
                        Some(RenderContext {
                            context: win
                        })
                    },
                    None => None
                }
            })
        }
    }
}

impl RenderContext
{
    pub fn make_current(&self)
    {
        self.context.make_context_current()
    }
}