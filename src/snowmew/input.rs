use glfw::{WindowEvent, Window, wait_events};

use std::util;
use std::comm::{Select};
use std::comm::{Chan, Port, SharedChan};

use std::trie::TrieMap;

pub struct InputState
{
    priv frames: ~[(f64, WindowEvent)]
}

pub type window_id = uint;

enum Command {
    AddPort(Port<(f64, WindowEvent)>, proc(window_id)),
    RemovePort(window_id, proc(bool))
}

fn wait_commands(cmd: &mut Port<Command>, ports: &mut TrieMap<Port<(f64, WindowEvent)>>) -> Command
{
    let mut handles = ~[];
    let select = Select::new();
    let mut cmd_handle = select.add(cmd);

    for (_, port) in ports.mut_iter() {
        handles.push(select.add(port));
    }

    loop {
        let id = select.wait();
        if id == cmd_handle.id {
            let cmd = cmd_handle.recv();
            match cmd {
                AddPort(port, reply) => return AddPort(port, reply),
                RemovePort(id, reply) => return RemovePort(id, reply),
            }
        }

        for h in handles.mut_iter() {
            if h.id == id {
                println!("{:?}", h.recv());
                // do event porcessing :o
                break;
            }
        }
    }
}

fn thread(cmd: Port<Command>)
{
    let mut max_id: window_id = 0;
    let mut cmd = cmd;
   
    let mut ports: TrieMap<Port<(f64, WindowEvent)>> = TrieMap::new();

    loop {
        let command = wait_commands(&mut cmd, &mut ports);

        match command {
            AddPort(port, reply) => {
                let id = max_id;
                max_id += 1;
                reply(max_id);

                ports.insert(id, port);
            },
            RemovePort(id, reply) => {
                reply(ports.remove(&id));
            }
        }
    }
}

#[deriving(Clone)]
pub struct InputManager
{
    priv cmd: SharedChan<Command>
}

impl InputManager
{
    pub fn new() -> InputManager
    {
        let (port, conn) = SharedChan::new();

        spawn(proc() {
            thread(port)
        });

        InputManager {
            cmd: conn
        }
    }

    pub fn add_window(&self, window: &mut Window) -> window_id
    {
        let (p, c) = Chan::new();
        let mut port = None;
        util::swap(&mut port, &mut window.event_port);
        
        self.cmd.send(AddPort(port.unwrap(), proc(id) c.send(id)));

        p.recv()
    }

    pub fn remove_window(&self, id: window_id) -> bool
    {
        let (p, c) = Chan::new();
        self.cmd.send(RemovePort(id, proc(succ) c.send(succ)));
        p.recv()
    }

    pub fn run(&self)
    {
        loop {
            wait_events();
        }
    }
}