use glfw::{WindowEvent, Window, wait_events};

use std::util;
use std::comm::{Select, Handle};
use std::comm::{Chan, Port, SharedChan};

use std::trie::TrieMap;

pub struct InputState
{
    priv frames: ~[(f64, WindowEvent)]
}

pub type window_id = uint;

enum Command {
    AddPort(Port<(f64, WindowEvent)>, proc(window_id)),
    RemovePort(window_id, proc(bool)),
    Finish(proc())
}


// This is a hack to make sure these are destroyed before the select is.
struct WaitState<'a> {
    cmd_handle: Handle<'a, Command>,
    handles: ~[Handle<'a, (f64, WindowEvent)>]
}

fn wait_commands(cmd: &mut Port<Command>, ports: &mut TrieMap<Port<(f64, WindowEvent)>>) -> Command
{
    let select = Select::new();

    let mut ws = WaitState {
        cmd_handle: select.add(cmd),
        handles: ~[]
    };

    for (_, port) in ports.mut_iter() {
        ws.handles.push(select.add(port));
    }

    loop {
        let id = select.wait();
        if id == ws.cmd_handle.id {
            let cmd = ws.cmd_handle.recv();
            match cmd {
                AddPort(port, reply) => {
                    return AddPort(port, reply);
                },
                RemovePort(id, reply) => {
                    return RemovePort(id, reply);
                },
                Finish(reply) => {  
                    return Finish(reply);
                }
            }
        }

        for h in ws.handles.mut_iter() {
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
                reply(id);

                ports.insert(id, port);
            },
            RemovePort(id, reply) => {
                reply(ports.remove(&id));
            },
            Finish(reply) => {
                reply();
                break;
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
        
        window.set_all_polling(true);
        self.cmd.send(AddPort(port.unwrap(), proc(id) c.send(id)));

        p.recv()
    }

    pub fn remove_window(&self, id: window_id) -> bool
    {
        let (p, c) = Chan::new();
        self.cmd.send(RemovePort(id, proc(succ) c.send(succ)));
        p.recv()
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
}