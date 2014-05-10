use std::task;
use std::mem;
use std::comm::Select;
use std::comm::{Sender, Receiver};
use std::comm::{Empty, Disconnected, Data};
use collections::HashSet;

use sync::{Arc, Mutex};
use glfw::{WindowEvent, Window, wait_events, Key, MouseButton, poll_events};
use glfw::{Press, Release, KeyEvent, MouseButtonEvent, CursorPosEvent};
use glfw::{CloseEvent, FocusEvent, EventReceiver};

use cgmath::quaternion::Quat;

use ovr;

pub type window_id = uint;

enum Command {
    AddReceiver(Receiver<(f64, WindowEvent)>, Arc<Mutex<Window>>, proc(window_id)),
    RemoveReceiver(window_id, proc(bool)),

    ResetOvr,

    Get(proc(InputState)),

    SetPos(window_id, (f64, f64)),

    Finish(proc()),

    Ack
}

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
    priv history: Option<Arc<InputHistory>>,
    priv keyboard: HashSet<Key>,
    priv mouse: HashSet<MouseButton>,
    priv should_close: bool,
    priv focus: bool,
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
}

struct OVR
{
    sensor: ovr::SensorFusion
}

struct WindowHandle
{
    id: uint,
    window: Arc<Mutex<Window>>,
    receiver: Receiver<(f64, WindowEvent)>,
}

struct ThreadState
{
    cmd: Receiver<Command>,
    state: InputState,
    max_id: uint,
    windows: ~[WindowHandle],
    ovr: Option<OVR>
}

impl ThreadState
{
    fn new(cmd: Receiver<Command>) -> ThreadState
    {
        ThreadState {
            cmd: cmd,
            state: InputState::new(),
            max_id: 1,
            windows: ~[],
            ovr: None
        }
    }

    fn wait(&mut self) -> Command
    {
        let select = Select::new();
        let mut cmd_handle = select.handle(&self.cmd);
        let mut win_handles = ~[];
        for win in self.windows.iter() {
            win_handles.push(select.handle(&win.receiver));
        }

        unsafe {  
            cmd_handle.add();
            for h in win_handles.mut_iter() { h.add() }
        }

        loop {
            select.wait();

            for p in self.windows.iter() {
                let dat = p.receiver.try_recv();
                match dat {
                    Empty | Disconnected => (),
                    Data((time, data)) => {
                        self.state.event(Some(time), data);
                    }
                }
            }

            match self.cmd.try_recv() {
                Empty | Disconnected => (),
                Data(cmd) => {
                    return cmd;
                }
            }

        }
    }

    fn add_window(&mut self, win: Arc<Mutex<Window>>, receiver: Receiver<(f64, WindowEvent)>) -> uint
    {
        let id = self.max_id;
        self.max_id += 1;
        self.windows.push(WindowHandle {
            id: id,
            window: win,
            receiver: receiver
        });
        id
    }

    fn remove(&mut self, id: uint) -> bool
    {
        let mut fidx = None;
        for (idx, win) in self.windows.iter().enumerate() {
            if win.id == id {
                fidx = Some(idx);
                break;
            }
        }

        if fidx.is_some() {
            self.windows.remove(fidx.unwrap());
            true
        } else {
            false
        }
    }

    fn set_pos(&mut self, id: uint, x: f64, y: f64)
    {
        for win in self.windows.iter() {
            if win.id == id {
                //win.window.access(|w| w.set_cursor_pos(x, y));
                //self.state.event(None, CursorPosEvent(x, y));
                return;
            }
        }
    }
}

fn thread(cmd: Receiver<Command>)
{
    let mut ts = ThreadState::new(cmd);

    loop {
        let cmd = ts.wait();

        match cmd {
            AddReceiver(receiver, window, reply) => {
                reply(ts.add_window(window, receiver));
            },
            SetPos(id, (x, y)) => {
                 ts.set_pos(id, x, y);
            }
            RemoveReceiver(id, reply) => {
                reply(ts.remove(id));
            },
            Finish(reply) => {
                reply();
                break;
            },
            ResetOvr => {
                match ts.ovr {
                    Some(ref ovr) => {
                        ovr.sensor.reset();
                    },
                    None => ()
                }
            },
            Get(reply) => {
                reply(ts.state.clone());
            },
            Ack => ()
        }
    }
}


pub struct InputManager
{
    priv cmd: Sender<Command>,
    priv ovr_sensor_device: Option<ovr::SensorDevice>,
    priv ovr_hmd_device: Option<ovr::HMDDevice>,
    priv ovr_device_manager: Option<ovr::DeviceManager>,
}

impl InputManager
{
    pub fn new() -> InputManager
    {
        let (sender, receiver) = channel();

        let task = task::task().named("input");

        // prime the channel to avoid a bug in Select
        sender.send(Ack);

        task.spawn(proc() {
            thread(receiver)
        });

        InputManager {
            cmd: sender,
            ovr_device_manager: None,
            ovr_hmd_device: None,
            ovr_sensor_device: None,
        }
    }

    pub fn add_window(&self, window: Arc<Mutex<Window>>, er: EventReceiver) -> InputHandle
    {
        let (send, recv) = channel();

        self.cmd.send(AddReceiver(er.to_receiver(), window, proc(id) send.send(id)));

        InputHandle {
            cmd: self.cmd.clone(),
            handle: recv.recv()
        }
    }

    pub fn wait(&self)
    {
        wait_events();
    }

    pub fn poll(&self)
    {
        poll_events();
    }

    pub fn finish(&self)
    {
        let (send, recv) = channel();
        self.cmd.send(Finish(proc() send.send(())));
        recv.recv()     
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
                fail!("todo");
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
    priv cmd: Sender<Command>,
    priv handle: window_id,
}

impl InputHandle
{
    pub fn get(&self) -> InputState
    {
        let (send, recv) = channel();
        self.cmd.send(Get(proc(state) send.send(state)));
        recv.recv()
    }

    pub fn set_cursor(&mut self, x: f64, y: f64)
    {
        self.cmd.send(SetPos(self.handle, (x, y)));
    }

    pub fn reset_ovr(&mut self) {
        self.cmd.send(ResetOvr)
    }
}