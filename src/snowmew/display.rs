use extra::arc::MutexArc;
use glfw::{Window, Windowed, FullScreen, Monitor};
use input::{InputManager, InputHandle};

pub struct Display
{
    priv window: MutexArc<Window>,
    priv handle: InputHandle,
}

impl Display
{
    pub fn new_window(im: &mut InputManager, size: (uint, uint)) -> Option<(Display, InputHandle)>
    {
        let (width, height) = size;
        let window = Window::create(width as u32, height as u32, "Snowmew", Windowed);
        let window = match window {
            Some(window) => window,
            None => return None
        };

        window.make_context_current();
        let window = MutexArc::new(window);
        let handle = im.add_window(window.clone());

        Some((Display {
            window: window,
            handle: handle.clone()
        },
        handle))
    }

    pub fn new_primary(im: &mut InputManager) -> Option<(Display, InputHandle)>
    {
        let primary = Monitor::get_primary().unwrap();
        let mode = primary.get_video_mode().unwrap();

        let window = Window::create(mode.width as u32, mode.height as u32, "Snowmew", FullScreen(primary));
        let window = match window {
            Some(window) => window,
            None => return None
        };

        window.make_context_current();
        let window = MutexArc::new(window);
        let handle = im.add_window(window.clone());

        Some((Display {
            window: window,
            handle: handle.clone()
        },
        handle))
    }

    pub fn new_ovr(im: &mut InputManager) -> Option<(Display, InputHandle)>
    {
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

        let window = Window::create(width as u32, height as u32, "Snowmew", FullScreen(monitors[idx]));
        let window = match window {
            Some(window) => window,
            None => return None
        };

        window.make_context_current();
        let window = MutexArc::new(window);
        let handle = im.add_window(window.clone());

        Some((Display {
            window: window,
            handle: handle.clone()
        },
        handle))
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