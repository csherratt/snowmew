//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use std::sync::Arc;
use std::kinds::Sync;
use std::sync::mpsc::Receiver;
#[cfg(target_os="linux")]
use libc::c_void;

use glfw::{WindowEvent, Glfw, Context, RenderContext};
use glfw::WindowMode::{Windowed, FullScreen};
use glfw;
use nice_glfw;

use collect::TrieMap;

use ovr;

use input;

pub type WindowId = uint;

struct WindowHandle {
    window: glfw::Window,
    forced_event: Option<WindowEvent>,
    receiver: Receiver<(f64, WindowEvent)>,
    title: String
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

        let (w, h) = window.get_framebuffer_size();

        self.windows.insert(id, {
            WindowHandle {
                window: window,
                forced_event: Some(glfw::WindowEvent::FramebufferSize(w, h)),
                receiver: recv,
                title: "snowmew".to_string()
            }
        });

        InputHandle{ handle: id }
    }

    fn create_window_context(&self, width: u32, height: u32, name: &str, mode: glfw::WindowMode)
            -> Option<(glfw::Window, Receiver<(f64,WindowEvent)>)> {

        nice_glfw::WindowBuilder::new(&self.glfw)
            .try_modern_context_hints()
            .size(width, height)
            .title(name)
            .mode(mode)
            .create()
    }

    pub fn window(&mut self, size: (u32, u32)) -> Option<Window> {
        let (width, height) = size;
        let win_opt = self.create_window_context(width, height, "Snowmew", Windowed);
        let (mut window, events) = match win_opt {
            Some((window, events)) => (window, events),
            None => return None
        };
        self.glfw.set_swap_interval(1);

        window.set_all_polling(true);
        window.show();
        let version = window.get_context_version();
        let rc = window.render_context();
        let handle = self.add_window(window, events);

        Some(Window {
            handle: handle,
            render: rc,
            version: (version.major, version.minor),
            hmd: None,
            os_spec: WindowOSSpec::new(&self.glfw)
        })
    }

    pub fn primary(&mut self, size: (u32, u32)) -> Option<Window> {
        let screen = {
            self.glfw.with_primary_monitor(|display| {
                let display = display.unwrap();
                let (width, height) = size;
                self.create_window_context(width, height, "Snowmew FullScreen", FullScreen(display))
            })
        };

        match screen {
            None => None,
            Some((mut window, events)) => {
                window.set_all_polling(true);
                window.show();
                let version = window.get_context_version();
                let rc = window.render_context();
                let handle = self.add_window(window, events);

                Some(Window {
                    handle: handle,
                    render: rc,
                    version: (version.major, version.minor),
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
                    let win_opt = self.create_window_context(width as u32, height as u32, "Snowmew FullScreen", FullScreen(m));
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
                let win_opt = self.create_window_context(width as u32, height as u32, "Snowmew FullScreen", FullScreen(m));
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
            version: (version.major, version.minor),
            hmd: Some(Arc::new(hmd)),
            os_spec: WindowOSSpec::new(&self.glfw)
        })
    }

    pub fn wait(&mut self) {
        self.glfw.wait_events();
    }

    pub fn poll(&mut self) {
        self.glfw.poll_events();
    }

    pub fn next_event(&mut self, handle: &InputHandle) -> input::EventGroup {
        let evt = self.windows.get_mut(&handle.handle)
        .map(|rx| {
            // this is a hack to inject the correct size into the event buffer
            match rx.forced_event.take() {
                Some(evt) => return input::Event::from_glfw(evt),
                None => ()
            };
            for (_, evt) in glfw::flush_messages(&rx.receiver) {
                let evt = input::Event::from_glfw(evt);
                if evt != input::EventGroup::Nop {
                    return evt;
                }
            }
            input::EventGroup::Nop
        });

        match evt {
            Some(e) => e,
            _ => input::EventGroup::Nop
        }
    }

    pub fn should_close(&mut self, handle: &InputHandle) -> bool {
        let should_close = self.windows.get_mut(&handle.handle)
            .map(|win| win.window.should_close());

        if let Some(x) = should_close {
            x
        } else {
            true
        }
    }

    pub fn set_title(&mut self, handle: &InputHandle, title: String) {
        self.windows.get_mut(&handle.handle)
            .map(|win| {
                if title != win.title {
                    win.window.set_title(title.as_slice());
                    win.title = title.clone();
                }
            });
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
        match self.windows.get_mut(&window.handle.handle) {
            Some(win) => win.window.set_pos(w, h),
            None => ()
        }
    }

    pub fn get_framebuffer_size(&mut self, window: &Window) -> (i32, i32) {
        match self.windows.get_mut(&window.handle.handle) {
            Some(win) => win.window.get_framebuffer_size(),
            None => (0, 0)
        }
    }

    pub fn get_proc_address(&self, name: &str) -> *const ::libc::c_void {
        self.glfw.get_proc_address_raw(name)
    }
}

#[derive(Clone, Copy)]
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
    display: *mut c_void
}

#[cfg(target_os="linux")]
impl WindowOSSpec {
    fn new(glfw: &Glfw) -> WindowOSSpec {
        WindowOSSpec {
            display: glfw.get_x11_display()
        }
    }
}

unsafe impl Send for WindowOSSpec {}

pub struct Window {
    handle: InputHandle,
    render: RenderContext,
    version: (uint, uint),
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
        self.version
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
    pub fn get_x11_display(&self) -> *mut c_void {
        self.os_spec.display
    }
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct IoState {
    pub render_size: (uint, uint),
    pub size: (uint, uint),
    pub position: (int, int),
    pub show_mouse: bool,
    pub mouse_over: bool,
    pub window_title: String
}

impl IoState {
    pub fn new() -> IoState {
        IoState {
            render_size: (1, 1),
            size: (1, 1),
            position: (0, 0),
            show_mouse: true,
            mouse_over: false,
            window_title: "snowmew".to_string()
        }
    }

    pub fn window_action(&mut self, win: input::WindowEvent) {
        match win {
            input::WindowEvent::Size(x, y) => {
                self.size = (x as uint, y as uint);
            }
            input::WindowEvent::Position(x, y) => {
                self.position = (x, y);
            }
            input::WindowEvent::MouseOver(mouse) => {
                self.mouse_over = mouse;
            }
        }
    }
}
