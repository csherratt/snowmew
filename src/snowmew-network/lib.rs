//   Copyright 2015 Colin Sherratt
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

extern crate "snowmew-core" as core;
extern crate "snowmew-position" as position;
extern crate "snowmew-graphics" as graphics;
extern crate "snowmew-render" as render;
extern crate "snowmew-input" as input;

extern crate wire;
extern crate bchannel;
extern crate bincode;
extern crate "rustc-serialize" as rustc_serialize;

use core::common::{Common, CommonData};
use position::{Positions, PositionData};
use graphics::{Graphics, GraphicsData};
use render::{Renderable, RenderData};
use input::{IoState, GetIoState};

use std::sync::mpsc;
use std::thread::Thread;
use std::ops::{Deref, DerefMut};
use wire::SizeLimit;
use wire::tcp::OutTcpStream;
use rustc_serialize::{Decodable, Encodable};

use core::game::Game;

#[derive(RustcEncodable, RustcDecodable)]
pub enum ServerMessage<T, E> {
    Image(T),
    Event(E),
    Sync
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum ClientMessage<E> {
    Event(E)
}

pub struct Server<Game, GameData> {
    game: Game,
    data: GameData
}

unsafe impl<T:Send, E:Send> Send for ServerClient<T, E> {}

struct ServerClient<T, E> {
    to_client: OutTcpStream<ServerMessage<T, E>>,
    from_client: bchannel::Receiver<ClientMessage<E>, bincode::DecodingError>
}

impl<G, GD:Clone+Send+Decodable+Encodable> Server<G, GD> {
    pub fn new(game: G, data: GD) -> Server<G, GD> {
        Server {
            game: game,
            data: data
        }
    }

    pub fn serve<E:Send+Encodable+Clone+Decodable>(mut self, iface: &str, port: u16)
        where G: Game<GD, E> {

        let (listener, _) = wire::listen_tcp(iface, port).unwrap();

        let (tx, rx): (mpsc::Sender<ServerClient<GD, E>>,
                       mpsc::Receiver<ServerClient<GD, E>>) = mpsc::channel();
        Thread::spawn(move || {
            let (read_limit, write_limit) = (SizeLimit::Infinite, SizeLimit::Infinite);

            for connection in listener.into_blocking_iter() {
                let (i, o) = wire::upgrade_tcp(connection, read_limit, write_limit);
                
                let client = ServerClient {
                    to_client: o,
                    from_client: i
                };

                tx.send(client);
            }
        });

        let mut clients = Vec::new();

        loop {
            // add clients to game
            while clients.len() == 0 {
                let mut res = rx.recv().ok().expect("failed to get message");
                res.to_client.send(&ServerMessage::Image(self.data.clone()));
                clients.push(res);
            }

            while let Ok(mut res) = rx.try_recv() {
                res.to_client.send(&ServerMessage::Image(self.data.clone()));
                clients.push(res)
            }

            let mut events = Vec::new();

            for c in clients.iter() {
                events.push(c.from_client.recv_block().expect("client sucks man"));
            }

            for ClientMessage::Event(e) in events.drain() {
                for c in clients.iter_mut() {
                    c.to_client.send(&ServerMessage::Event(e.clone()));
                }
                self.data = self.game.step(e, self.data)
            }
            for c in clients.iter_mut() {
                c.to_client.send(&ServerMessage::Sync);
            }

        }
    }
}

#[derive(Clone)]
pub struct ClientState<T, E> {
    sync: bool,
    predict_frame: u32,
    predict: T,
    predict_delta: Vec<(u32, E)>,
    server_frame: u32,
    server: T
}

impl<T, E> Deref for ClientState<T, E> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        &self.predict
    }
}

impl<T, E> DerefMut for ClientState<T, E> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.predict
    }
}

impl<E, T: Common> Common for ClientState<T, E> {
    fn get_common<'a>(&'a self) -> &'a CommonData { self.predict.get_common() }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { self.predict.get_common_mut() }
}

impl<E, T: Positions> Positions for ClientState<T, E> {
    fn get_position<'a>(&'a self) -> &'a PositionData { self.predict.get_position() }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { self.predict.get_position_mut() }
}

impl<E, T: Graphics> Graphics for ClientState<T, E> {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData { self.predict.get_graphics() }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData { self.predict.get_graphics_mut() }
}

impl<E, T: Renderable> Renderable for ClientState<T, E> {
    fn get_render_data<'a>(&'a self) -> &'a RenderData { self.predict.get_render_data() }
    fn get_render_data_mut<'a>(&'a mut self) -> &'a mut RenderData { self.predict.get_render_data_mut() }
}

impl<E, T: GetIoState> GetIoState for ClientState<T, E> {
    fn get_io_state<'a>(&'a self) -> &'a IoState { self.predict.get_io_state() }
    fn get_io_state_mut<'a>(&'a mut self) -> &'a mut IoState { self.predict.get_io_state_mut() }
}

pub struct Client<G, T, E> {
    game: G,
    to_server: OutTcpStream<ClientMessage<E>>,
    from_server: bchannel::Receiver<ServerMessage<T, E>, bincode::DecodingError>
}

impl<G, T:Send+Encodable+Decodable+Clone, E:Send+Encodable+Decodable> Client<G, T, E> {
    pub fn new(game: G, server: &str, port: u16) -> Client<G, T, E> {
        let (o, i) = wire::connect_tcp(server, port, SizeLimit::Infinite, SizeLimit::Infinite).unwrap();

        Client {
            game: game,
            to_server: i,
            from_server: o  
        }
    }

    pub fn gamedata(&mut self) -> ClientState<T, E> {
        match self.from_server.recv_block().expect("failed to get state") {
            ServerMessage::Image(state) => {
                println!("download done");
                ClientState {
                    sync: false,
                    predict_frame: 0,
                    predict: state.clone(),
                    predict_delta: Vec::new(),
                    server_frame: 0,
                    server: state.clone()
                }
            },
            _ => panic!("expected game data")
        }
    }
}

impl<G: Game<T, E>, T:Send+Encodable+Decodable+Clone, E:Send+Encodable+Decodable+Clone>
    Game<ClientState<T, E>, E> for Client<G, T, E> {

    fn step(&mut self, event: E, mut gd: ClientState<T, E>) -> ClientState<T, E> {
        let mut frame = gd.predict.clone();
        let mut index = gd.predict_frame;

        loop {
            match self.from_server.recv() {
                Some(ServerMessage::Image(_)) => panic!("unexpected game data"),
                Some(ServerMessage::Event(e)) => {
                    gd.server = self.game.step(e, gd.server);
                }
                Some(ServerMessage::Sync) => {
                    gd.server_frame += 1;
                    frame = gd.server.clone();
                    index = gd.server_frame;
                },
                None => break
            }
        }

        if gd.sync {
            gd.sync = gd.predict_frame > gd.server_frame + 1;
            if gd.sync {
                return gd
            }
        } else if gd.predict_frame > gd.server_frame + 5 {
            gd.sync = true;
            return gd;
        }

        self.to_server.send(&ClientMessage::Event(event.clone()));
        gd.predict_delta.push((gd.predict_frame, event));
        gd.predict_frame += 1;

        let mut predict = Vec::new();
        for (idx, e) in gd.predict_delta.drain() {
            if idx > index {
                println!("{} > {}", idx, index);
                frame = self.game.step(e.clone(), frame);
            }

            if idx > gd.server_frame {
                predict.push((idx, e));
            }
        }

        gd.predict = frame;
        gd.predict_delta = predict;
        gd
    }
}

