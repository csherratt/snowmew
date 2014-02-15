use sync::{CowArc, MutexArc};
use glfw::{WindowEvent, Window, wait_events, Key, MouseButton};
use glfw::{Press, Release, KeyEvent, MouseButtonEvent, CursorPosEvent};
use glfw::{CloseEvent, FocusEvent};
use std::mem;
use std::comm::Select;
use std::comm::{Chan, Port};

use std::trie::TrieMap;
use std::hashmap::HashSet;

use cgmath::quaternion::Quat;

use ovr;

pub type window_id = uint;

enum Command {
    AddPort(Port<(f64, WindowEvent)>, MutexArc<Window>, proc(window_id)),
    RemovePort(window_id, proc(bool)),

    AddOVR(Port<ovr::Message>),
    ResetOvr,

    Get(proc(InputState)),

    SetPos(window_id, (f64, f64)),

    Finish(proc())
}

#[deriving(Clone)]
struct InputHistory
{
    older: Option<CowArc<InputHistory>>,
    time: Option<f64>,
    event: WindowEvent
}

#[deriving(Clone)]
pub struct InputState
{
    priv history: Option<CowArc<InputHistory>>,
    priv keyboard: HashSet<Key>,
    priv mouse: HashSet<MouseButton>,
    priv should_close: bool,
    priv focus: bool,
    predicted: Quat<f32>,
}

struct InputHistoryIterator
{
    current: Option<CowArc<InputHistory>>
}

impl Iterator<(Option<f64>, WindowEvent)> for InputHistoryIterator
{
    fn next(&mut self) -> Option<(Option<f64>, WindowEvent)>
    {
        let (next, res) = match self.current {
            Some(ref next) => {
                let next = next.get();
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
    fn new() -> InputState
    {
        InputState {
            history: None,
            keyboard: HashSet::new(),
            mouse: HashSet::new(),
            should_close: false,
            focus: false,
            predicted: Quat::identity()
        }
    }

    fn event(&mut self, time: Option<f64>, event: WindowEvent)
    {
        self.history = Some(CowArc::new( InputHistory{
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
}

struct OVR
{
    port: Port<ovr::Message>,
    sensor: ovr::SensorFusion
}

fn wait_commands(state: &mut InputState,
                 cmd: &mut Port<Command>,
                 ovr: &mut Option<OVR>,
                 ports: &mut TrieMap<(MutexArc<Window>, Port<(f64, WindowEvent)>)>) -> Command
{
    let select = Select::new();

    let mut handles = ~[];
    let mut cmd_handle = select.handle(cmd);
    let (mut ovr_handle, sf) = match *ovr {
        Some(ref mut ovr) => (Some(select.handle(&ovr.port)), Some(&ovr.sensor)),
        None => (None, None)
    };

    for (id, &(_, ref mut port)) in ports.mut_iter() {
        handles.push((id, select.handle(port)));
    }

    loop {
        let id = select.wait();
        if cmd_handle.id() == id {
            let cmd = cmd_handle.recv();
            match cmd {
                AddPort(port, window, reply) => {
                    return AddPort(port, window, reply);
                },
                AddOVR(port) => {
                    return AddOVR(port);
                },
                ResetOvr => {
                    return ResetOvr;
                },
                RemovePort(id, reply) => {
                    return RemovePort(id, reply);
                },
                SetPos(id, pos) => {
                    return SetPos(id, pos);
                },
                Get(reply) => {
                    reply(state.clone());
                },
                Finish(reply) => {  
                    return Finish(reply);
                }
            }
        }

        for &(_, ref mut handle) in handles.mut_iter() {
            if handle.id() == id {
                let (time, event) = handle.recv();
                state.event(Some(time), event);
                break;
            }
        }

        match ovr_handle {
            Some(ref mut ovr_handle) => {
                if ovr_handle.id() == id {
                    let msg = ovr_handle.recv();
                    match sf {
                        Some(ref sf) => {
                            sf.on_message(&msg);
                            state.predicted = sf.get_predicted_orientation(None);
                        },
                        _ => (),
                    }
                }
            },
            None => ()
        }
    }
}

fn thread(cmd: Port<Command>)
{
    let mut state = InputState::new();
    let mut max_id: window_id = 0;
    let mut cmd = cmd;
    
    let mut ovr: Option<OVR> = None;
    let mut ports: TrieMap<(MutexArc<Window>, Port<(f64, WindowEvent)>)> = TrieMap::new();

    loop {
        let command = wait_commands(&mut state, &mut cmd, &mut ovr, &mut ports);

        match command {
            AddPort(port, window, reply) => {
                let id = max_id;
                max_id += 1;
                reply(id);

                ports.insert(id, (window, port));
            },
            AddOVR(port) => {
                ovr = Some(OVR {
                    port: port,
                    sensor: ovr::SensorFusion::new().unwrap(),
                });
            },
            SetPos(id, (x, y)) => {
                 match ports.find(&id) {
                    Some(&(ref window, _)) => {
                        unsafe {window.unsafe_access(|w| w.set_cursor_pos(x, y))}
                        state.event(None, CursorPosEvent(x, y));
                    },
                    None => ()
                }
            }
            RemovePort(id, reply) => {
                reply(ports.remove(&id));
            },
            Finish(reply) => {
                reply();
                break;
            },
            ResetOvr => {
                match ovr {
                    Some(ref ovr) => {
                        ovr.sensor.reset();
                    },
                    None => ()
                }
            }
            Get(_) => (),
        }
    }
}


pub struct InputManager
{
    priv cmd: Chan<Command>,
    priv ovr_sensor_device: Option<ovr::SensorDevice>,
    priv ovr_hmd_device: Option<ovr::HMDDevice>,
    priv ovr_device_manager: Option<ovr::DeviceManager>,
}

impl InputManager
{
    pub fn new() -> InputManager
    {
        let (port, conn) = Chan::new();

        spawn(proc() {
            thread(port)
        });

        InputManager {
            cmd: conn,
            ovr_device_manager: None,
            ovr_hmd_device: None,
            ovr_sensor_device: None,
        }
    }

    pub fn add_window(&self, window: MutexArc<Window>) -> InputHandle
    {
        let (p, c) = Chan::new();
        let mut port = None;

        unsafe {
            window.unsafe_access(|window| {
                mem::swap(&mut port, &mut window.event_port);
                window.set_all_polling(true);
            });
        }
        
        self.cmd.send(AddPort(port.unwrap(), window, proc(id) c.send(id)));

        InputHandle {
            cmd: self.cmd.clone(),
            handle: p.recv()
        }
    }

    pub fn wait(&self)
    {
        wait_events();
    }

    pub fn finish(&self)
    {
        let (p, c) = Chan::new();
        self.cmd.send(Finish(proc() c.send(())));
        p.recv()     
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
            Some(ref mut sd) => {
                let (port, chan) = Chan::new();
                sd.register_chan(~chan);
                self.cmd.send(AddOVR(port));
            },
            None => return false
        }

        true
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
    priv cmd: Chan<Command>,
    priv handle: window_id,
}

impl InputHandle
{
    pub fn get(&self) -> InputState
    {
        let (p, c) = Chan::new();
        self.cmd.send(Get(proc(state) c.send(state)));
        p.recv()
    }

    pub fn set_cursor(&mut self, x: f64, y: f64)
    {
        self.cmd.send(SetPos(self.handle, (x, y)));
    }

    pub fn reset_ovr(&mut self) {
        self.cmd.send(ResetOvr)
    }
}