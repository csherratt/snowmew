use sync::Arc;
use glfw::{WindowEvent, Key, MouseButton, Glfw, Context};
use glfw::{Press, Release, KeyEvent, MouseButtonEvent, CursorPosEvent};
use glfw::{CloseEvent, FocusEvent};
use glfw::{Windowed, RenderContext};
use glfw;
use gl;

use semver;
use collections::{HashSet, TrieMap};

use cgmath::quaternion::Quat;

use ovr;

pub type WindowId = uint;

#[deriving(Clone)]
struct InputHistory
{
    older: Option<Arc<InputHistory>>,
    time: Option<f64>,
    event: WindowEvent
}

#[deriving(Clone)]
pub struct InputState
{
    history: Option<Arc<InputHistory>>,
    keyboard: HashSet<Key>,
    mouse: HashSet<MouseButton>,
    should_close: bool,
    focus: bool,
    framebuffer_size: (i32, i32),
    screen_size: (i32, i32),
    predicted: Quat<f32>,
}

struct InputHistoryIterator
{
    current: Option<Arc<InputHistory>>
}

impl Iterator<(Option<f64>, WindowEvent)> for InputHistoryIterator
{
    fn next(&mut self) -> Option<(Option<f64>, WindowEvent)>
    {
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

impl InputState
{
    fn new(win: &glfw::Window) -> InputState
    {
        InputState {
            history: None,
            keyboard: HashSet::new(),
            mouse: HashSet::new(),
            should_close: false,
            focus: win.is_focused(),
            framebuffer_size: win.get_framebuffer_size(),
            screen_size: win.get_size(),
            predicted: Quat::identity()
        }
    }

    fn event(&mut self, time: Option<f64>, event: WindowEvent)
    {
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

    fn iter(&self) -> InputHistoryIterator
    {
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

    pub fn time(&self) -> f64
    {
        for (t, _) in self.iter() {
            match t {
                Some(t) => return t,
                None => ()
            }
        }
        0.
    }

    pub fn cursor_delta(&self, epoc: f64) -> Option<(f64, f64)>
    {
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

    pub fn should_close(&self) -> bool
    {
        self.should_close
    }

    pub fn is_focused(&self) -> bool
    {
        self.focus
    }

    pub fn screen_size(&self) -> (i32, i32)
    {
        self.screen_size.clone()
    }

    pub fn framebuffer_size(&self) -> (i32, i32)
    {
        self.framebuffer_size.clone()
    }
}

struct WindowHandle
{
    window: glfw::Window,
    receiver: Receiver<(f64, WindowEvent)>,
    state: InputState
}

pub struct IOManager
{
    glfw: Glfw,
    ovr_sensor_device: Option<ovr::SensorDevice>,
    ovr_hmd_device: Option<ovr::HMDDevice>,
    ovr_device_manager: Option<ovr::DeviceManager>,
    windows: TrieMap<WindowHandle>,
    window_id: uint
}

pub struct Window
{
    handle: InputHandle,
    render: RenderContext,
    version: semver::Version
}

impl IOManager
{
    pub fn new(glfw: glfw::Glfw) -> IOManager
    {
        IOManager {
            glfw: glfw,
            ovr_device_manager: None,
            ovr_hmd_device: None,
            ovr_sensor_device: None,
            windows: TrieMap::new(),
            window_id: 0
        }
    }

    fn add_window(&mut self, window: glfw::Window, recv: Receiver<(f64, WindowEvent)>) -> InputHandle
    {
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

    pub fn window(&mut self, size: (u32, u32)) -> Option<Window>
    {
        let (width, height) = size;
        let win_opt = self.glfw.create_window(width, height, "Snowmew", Windowed);
        let (mut window, events) = match win_opt {
            Some((window, events)) => (window, events),
            None => return None
        };

        println!("{:?}", window.get_context_version());

        window.make_current();
        gl::load_with(|name| self.glfw.get_proc_address(name));
        glfw::make_context_current(None);

        window.set_all_polling(true);
        window.show();
        let version = window.get_context_version();
        let rc = window.render_context();
        let handle = self.add_window(window, events);

        Some(Window {
            handle: handle,
            render: rc,
            version: version
        })
    }

    pub fn get(&self, handle: &InputHandle) -> InputState
    {
        self.windows.find(&handle.handle).unwrap().state.clone()
    }

    fn update(&mut self)
    {
        for (_, win) in self.windows.mut_iter() {
            for (time, event) in glfw::flush_messages(&win.receiver) {
                win.state.event(Some(time), event);
            }
        }
    }

    pub fn wait(&mut self)
    {
        self.glfw.wait_events();
        self.update();
    }

    pub fn poll(&mut self)
    {
        self.glfw.poll_events();
        self.update();
    }

    pub fn setup_ovr(&mut self) -> bool
    {
        if self.ovr_device_manager.is_some() &&
           self.ovr_sensor_device.is_some() &&
           self.ovr_hmd_device.is_some() {
            return true;
        }

        if self.ovr_device_manager.is_none() {
            ovr::init();
            self.ovr_device_manager = ovr::DeviceManager::new();
        }

        match self.ovr_device_manager {
            Some(ref hmd) => {
                self.ovr_hmd_device = hmd.enumerate();
            },
            None => return false
        }

        match self.ovr_hmd_device {
            Some(ref hmd) => {
                self.ovr_sensor_device = hmd.get_sensor();
            },
            None => return false
        }

        match self.ovr_sensor_device {
            Some(_) => {
                fail!("todo");
            },
            None => return false
        }
    }

    pub fn ovr_manager<'a>(&'a mut self) -> Option<&'a ovr::DeviceManager>
    {
        if self.ovr_device_manager.is_none() {
            ovr::init();
            self.ovr_device_manager = ovr::DeviceManager::new();
        }
        self.ovr_device_manager.as_ref()
    }
}

#[deriving(Clone)]
pub struct InputHandle
{
    handle: uint,
}

impl Window
{
    pub fn swap_buffers(&self)
    {
        self.render.swap_buffers()
    }

    pub fn make_context_current(&self)
    {
        self.render.make_current()
    }

    pub fn get_context_version(&self) -> (uint, uint)
    {
        let version = self.version.clone();
        (version.major, version.minor)
    }

    pub fn is_hmd(&self) -> bool
    {
        false
    }

    pub fn handle(&self) -> InputHandle
    {
        self.handle.clone()
    }
}