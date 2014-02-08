use sync::MutexArc;
use glfw::{Window, Windowed, FullScreen, Monitor, WindowMode};
use input::{InputManager, InputHandle};

use glfw;
use gl;

pub struct Display
{
    priv window: MutexArc<Window>,
    priv handle: InputHandle,
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

        window.make_context_current();
        gl::load_with(glfw::get_proc_address);

        let window = MutexArc::new(window);
        let handle = im.add_window(window.clone());

        Some((Display {
            window: window,
            handle: handle.clone()
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
            Some(win) => {
                Some(win)
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

}