use sync::Arc;
#[cfg(target_os="linux")]
use libc::c_void;

use glfw::{WindowEvent, Key, MouseButton, Glfw, Context};
use glfw::{Press, Release, KeyEvent, MouseButtonEvent, CursorPosEvent};
use glfw::{CloseEvent, FocusEvent, FullScreen};
use glfw::{Windowed, RenderContext};
use glfw;
use gl;

use semver;
use std::collections::HashSet;
use collections::TrieMap;

use cgmath::quaternion::Quaternion;

use ovr;

pub type WindowId = uint;

#[cfg(target_os="macos")]
static OS_GL_MINOR_MAX: u32 = 1;

#[cfg(target_os="linux")]
static OS_GL_MINOR_MAX: u32 = 4;

#[deriving(Clone)]
struct InputHistory {
    older: Option<Arc<InputHistory>>,
    time: Option<f64>,
    event: WindowEvent
}

#[deriving(Clone)]
pub struct InputState {
    history: Option<Arc<InputHistory>>,
    keyboard: HashSet<Key>,
    mouse: HashSet<MouseButton>,
    should_close: bool,
    focus: bool,
    framebuffer_size: (i32, i32),
    screen_size: (i32, i32),
    predicted: Quaternion<f32>,
}

struct InputHistoryIterator {
    current: Option<Arc<InputHistory>>
}

impl Iterator<(Option<f64>, WindowEvent)> for InputHistoryIterator {
    fn next(&mut self) -> Option<(Option<f64>, WindowEvent)> {
        let (next, res) = match self.current {
            Some(ref next) => {
                let next = next.deref();
                (next.older.clone(), Some((next.time.clone(), next.event.clone())))
            },
            None => (None, None)
        };

        self.current = next;
        res
    }
}

pub struct DeltaIterator {
    current: Option<Arc<InputHistory>>,
    origin: Option<Arc<InputHistory>>
}


impl Iterator<(Option<f64>, WindowEvent)> for DeltaIterator {
    fn next(&mut self) -> Option<(Option<f64>, WindowEvent)> {
        match (&self.current, &self.origin) {
            (&None, &None) => return None,
            (&Some(ref a), &Some(ref b)) => {
                if a.deref() as *InputHistory == b.deref() as *InputHistory {
                    return None
                }
            }
            _ => ()
        }

        let (next, res) = match self.current {
            Some(ref next) => {
                let next = next.deref();
                (next.older.clone(), Some((next.time.clone(), next.event.clone())))
            },
            None => (None, None)
        };

        self.current = next;
        res
    }
}

impl InputState {
    fn new(win: &glfw::Window) -> InputState {
        InputState {
            history: None,
            keyboard: HashSet::new(),
            mouse: HashSet::new(),
            should_close: false,
            focus: win.is_focused(),
            framebuffer_size: win.get_framebuffer_size(),
            screen_size: win.get_size(),
            predicted: Quaternion::identity()
        }
    }

    fn event(&mut self, time: Option<f64>, event: WindowEvent) {
        self.history = Some(Arc::new( InputHistory{
            older: self.history.clone(),
            time: time,
            event: event.clone()
        }));

        match event {
            KeyEvent(key, _, Press, _) => { self.keyboard.insert(key); },
            KeyEvent(key, _, Release, _) => { self.keyboard.remove(&key); },
            MouseButtonEvent(key, Press, _) => { self.mouse.insert(key); },
            MouseButtonEvent(key, Release, _) => { self.mouse.remove(&key); },
            CloseEvent => { self.should_close = true; },
            FocusEvent(s) => { self.focus = s; },
            _ => ()
        }
    }

    fn iter(&self) -> InputHistoryIterator {
        InputHistoryIterator {
            current: self.history.clone()
        }
    }

    pub fn key_down(&self, key: Key) -> bool
    {
        self.keyboard.contains(&key)
    }

    pub fn mouse_up(&self, button: MouseButton) -> bool
    {
        self.mouse.contains(&button)
    }

    pub fn time(&self) -> f64 {
        for (t, _) in self.iter() {
            match t {
                Some(t) => return t,
                None => ()
            }
        }
        0.
    }

    pub fn cursor_delta(&self, epoc: f64) -> Option<(f64, f64)> {
        let mut latest = None;
        let mut old = (0f64, 0f64);
        let mut iter = self.iter();

        // find the latest cursor position
        for (time, event) in iter {
            // no change found
            if time.is_none() || time.unwrap() <= epoc {
                return None;
            }
            match event {
                CursorPosEvent(x, y) => {
                    latest = Some((x, y));
                    break;
                },
                _ => ()
            }
        }

        // no change found
        if latest.is_none() {
            return None;
        }

        let (nx, ny) = latest.unwrap();

        // find the first cursor positon before
        for (time, event) in iter {
            if time.is_none() || time.unwrap() <= epoc {
                match event {
                    CursorPosEvent(x, y) => {
                        old = (x, y);
                        break;
                    },
                    _ => ()
                }
            }
        }

        let (x, y) = old;
        Some((nx-x, ny-y))
    }

    pub fn should_close(&self) -> bool {
        self.should_close
    }

    pub fn is_focused(&self) -> bool {
        self.focus
    }

    pub fn screen_size(&self) -> (i32, i32) {
        self.screen_size.clone()
    }

    pub fn framebuffer_size(&self) -> (i32, i32) {
        self.framebuffer_size.clone()
    }

    pub fn iter_delta(&self, old: &InputState) -> DeltaIterator {
        DeltaIterator {
            current: self.history.clone(),
            origin: old.history.clone()
        }
    }
}

struct WindowHandle {
    window: glfw::Window,
    receiver: Receiver<(f64, WindowEvent)>,
    state: InputState
}

pub struct IOManager {
    glfw: Glfw,
    ovr: Option<ovr::Ovr>,
    windows: TrieMap<WindowHandle>,
    window_id: uint
}

impl IOManager {
    pub fn new(glfw: glfw::Glfw) -> IOManager {
        IOManager {
            glfw: glfw,
            ovr: None,
            windows: TrieMap::new(),
            window_id: 0
        }
    }

    fn add_window(&mut self, window: glfw::Window, recv: Receiver<(f64, WindowEvent)>) -> InputHandle {
        let id = self.window_id;
        self.window_id += 1;

        let state = InputState::new(&window);

        self.windows.insert(id, {
            WindowHandle {
                window: window,
                receiver: recv,
                state: state
            }
        });

        InputHandle{ handle: id }
    }

    fn create_window_context(&self, width: u32, height: u32, name: &str, mode: glfw::WindowMode) 
            -> Option<(glfw::Window, Receiver<(f64,WindowEvent)>)> {

        self.glfw.window_hint(glfw::ContextVersion(4, OS_GL_MINOR_MAX));
        let window = self.glfw.create_window(width, height, name, mode);
        if window.is_some() {
            return window;
        }

        None
    }

    pub fn window(&mut self, size: (u32, u32)) -> Option<Window> {
        let (width, height) = size;
        let win_opt = self.create_window_context(width, height, "Snowmew", Windowed);
        let (mut window, events) = match win_opt {
            Some((window, events)) => (window, events),
            None => return None
        };

        window.make_current();
        gl::load_with(|name| self.glfw.get_proc_address(name));
        self.glfw.set_swap_interval(0);
        glfw::make_context_current(None);

        window.set_all_polling(true);
        window.show();
        let version = window.get_context_version();
        let rc = window.render_context();
        let handle = self.add_window(window, events);

        Some(Window {
            handle: handle,
            render: rc,
            version: version,
            hmd: None,
            os_spec: WindowOSSpec::new(&self.glfw)
        })
    }

    pub fn primary(&mut self, size: (u32, u32)) -> Option<Window> {
        let screen = {
            self.glfw.with_primary_monitor(|display| {
                let display = display.unwrap();
                let (width, height) = size;
                self.create_window_context(width, height, "Snowmew Fullscreen", FullScreen(display))
            })
        };

        match screen {
            None => None,
            Some((mut window, events)) => {
                window.make_current();
                gl::load_with(|name| self.glfw.get_proc_address(name));
                self.glfw.set_swap_interval(0);
                glfw::make_context_current(None);

                window.set_all_polling(true);
                window.show();
                let version = window.get_context_version();
                let rc = window.render_context();
                let handle = self.add_window(window, events);

                Some(Window {
                    handle: handle,
                    render: rc,
                    version: version,
                    hmd: None,
                    os_spec: WindowOSSpec::new(&self.glfw)
                })
            }
        }

    }

    pub fn get_primary_resolution(&self) -> (u32, u32) {
        self.glfw.with_primary_monitor(|display| {
            let display = display.expect("Could not get primnay display");
            let vm = display.get_video_mode().expect("Could not get video mode");
            (vm.width, vm.height)
        })
    }

    pub fn get_primary_position(&self) -> (i32, i32) {
        self.glfw.with_primary_monitor(|display| {
            let display = display.expect("Could not get primnay display");
            display.get_pos()
        })
    }

    #[cfg(target_os="linux")]
    fn create_hmd_window(&self, hmd: &ovr::HmdDescription) -> Option<(glfw::Window, Receiver<(f64,WindowEvent)>)> {
        self.glfw.with_connected_monitors(|monitors| {
            for m in monitors.iter() {
                let (x, y) = m.get_pos();
                if x == hmd.window_position.x && 
                   y == hmd.window_position.y {
                    let (width, height) = (hmd.resolution.x, hmd.resolution.y);
                    let win_opt = self.create_window_context(width as u32, height as u32, "Snowmew Fullscreen", FullScreen(m));
                    let (window, events) = match win_opt {
                        Some((window, events)) => (window, events),
                        None => return None
                    };

                    return Some((window, events));
                }
            }

            // fallback if we could not guess at the screen
            let (width, height) = (hmd.resolution.x, hmd.resolution.y);
            let win_opt = self.glfw.create_window(width as u32, height as u32, "Snowmew", Windowed);
            let (window, events) = match win_opt {
                Some((window, events)) => (window, events),
                None => return None
            };

            // move viewport
            let (dx, dy) = (hmd.window_position.x, hmd.window_position.y);
            window.set_pos(dx as i32, dy as i32);

            Some((window, events))

        })
    }

    #[cfg(target_os="macos")]
    fn create_hmd_window(&self, hmd: &ovr::HmdDescription) -> Option<(glfw::Window, Receiver<(f64,WindowEvent)>)> {
        self.glfw.with_connected_monitors(|monitors| {
            for m in monitors.iter() {
                if !m.get_name().as_slice().contains("Rift") {
                    continue;
                }

                let (width, height) = (hmd.resolution.x, hmd.resolution.y);
                let win_opt = self.create_window_context(width as u32, height as u32, "Snowmew Fullscreen", FullScreen(m));
                let (window, events) = match win_opt {
                    Some((window, events)) => (window, events),
                    None => return None
                };

                return Some((window, events));
            }
            None
        })
    }

    pub fn hmd(&mut self) -> Option<Window> {
        if !self.setup_ovr() {
            return None;
        }

        let (window, events, rc, hmd) = {
            let hmd = match self.ovr.as_ref().unwrap().first_hmd() {
                Some(hmd) => hmd,
                None => return None
            };
            let hmdinfo = hmd.get_description();

            let (mut window, events) = match self.create_hmd_window(&hmdinfo) {
                Some((window, events)) => (window, events),
                None => return None
            };

            window.make_current();
            gl::load_with(|name| self.glfw.get_proc_address(name));
            self.glfw.set_swap_interval(1);
            glfw::make_context_current(None);

            window.set_all_polling(true);
            window.show();

            let rc = window.render_context();
            (window, events, rc, hmd)
        };

        let version = window.get_context_version();
        let handle = self.add_window(window, events);

        Some(Window {
            handle: handle,
            render: rc,
            version: version,
            hmd: Some(Arc::new(hmd)),
            os_spec: WindowOSSpec::new(&self.glfw)
        })
    }

    pub fn get(&self, handle: &InputHandle) -> InputState {
        self.windows.find(&handle.handle).unwrap().state.clone()
    }

    fn update(&mut self) {
        for (_, win) in self.windows.mut_iter() {
            for (time, event) in glfw::flush_messages(&win.receiver) {
                win.state.event(Some(time), event);
            }
        }
    }

    pub fn wait(&mut self) {
        self.glfw.wait_events();
        self.update();
    }

    pub fn poll(&mut self) {
        self.glfw.poll_events();
        self.update();
    }

    fn setup_ovr(&mut self) -> bool {
        if self.ovr.is_some() &&
           self.ovr.as_ref().unwrap().detect() > 0 {
            return true;
        }

        if self.ovr.is_none() {
            self.ovr = ovr::Ovr::init();
        }

        self.ovr.is_some() && self.ovr.as_ref().unwrap().detect() > 0
    }

    pub fn set_window_position(&mut self, window: &Window, pos: (i32, i32)) {
        let (w, h) = pos;
        match self.windows.find_mut(&window.handle.handle) {
            Some(win) => win.window.set_pos(w, h),
            None => ()
        }
    }

    pub fn get_framebuffer_size(&mut self, window: &Window) -> (i32, i32) {
        match self.windows.find_mut(&window.handle.handle) {
            Some(win) => win.window.get_framebuffer_size(),
            None => (0, 0)
        }
    }
}

#[deriving(Clone)]
pub struct InputHandle {
    handle: uint,
}

#[cfg(target_os="macos")]
struct WindowOSSpec;

#[cfg(target_os="macos")]
impl WindowOSSpec {
    fn new(_: &Glfw) -> WindowOSSpec {WindowOSSpec}
}

#[cfg(target_os="linux")]
struct WindowOSSpec {
    display: *c_void
}

#[cfg(target_os="linux")]
impl WindowOSSpec {
    fn new(glfw: &Glfw) -> WindowOSSpec {
        WindowOSSpec {
            display: glfw.get_x11_display()
        }
    }
}

pub struct Window {
    handle: InputHandle,
    render: RenderContext,
    version: semver::Version,
    hmd: Option<Arc<ovr::Hmd>>,
    os_spec: WindowOSSpec
}

impl Window {
    pub fn swap_buffers(&self) {
        self.render.swap_buffers()
    }
    
    pub fn make_context_current(&self) {
        self.render.make_current()
    }

    pub fn get_context_version(&self) -> (uint, uint) {
        let version = self.version.clone();
        (version.major, version.minor)
    }

    pub fn handle(&self) -> InputHandle {
        self.handle.clone()
    }

    pub fn is_hmd(&self) -> bool {
        self.hmd.is_some()
    }

    pub fn get_hmd<'a>(&'a self) -> Arc<ovr::Hmd> {
        self.hmd.as_ref().expect("no hmd device found!").clone()
    }

    /// Wrapper for `glfwGetGLXContext`
    #[cfg(target_os="linux")]
    pub fn get_x11_display(&self) -> *c_void {
        self.os_spec.display
    }

}