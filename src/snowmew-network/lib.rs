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
use std::iter::FromIterator;
use std::old_io::net::ip::ToSocketAddr;
use wire::SizeLimit;
use wire::tcp::OutTcpStream;
use rustc_serialize::{Decodable, Encodable};

use core::game::Game;

#[derive(RustcEncodable, RustcDecodable)]
pub enum ServerMessage<T, E> {
    Image(T),
    Event(E)
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum ClientMessage<E> {
    Join,
    Event(E)
}

pub struct Server<Game, GameData> {
    game: Game,
    data: GameData
}

unsafe impl<T:Send, E:Send, CE:Send> Send for ServerClient<T, E, CE> {}

struct ServerClient<T, E, CE> {
    joined: bool,
    to_client: mpsc::Sender<ServerMessage<T, CE>>,
    from_client: bchannel::Receiver<ClientMessage<E>, bincode::DecodingError>
}

impl<G, GD:Clone+Send+Decodable+Encodable> Server<G, GD> {
    pub fn new(game: G, data: GD) -> Server<G, GD> {
        Server {
            game: game,
            data: data
        }
    }

    pub fn serve<E: Send+Encodable+Clone+Decodable+FromIterator<CE>,
                 CE: Send+Encodable+Clone+Decodable,
                 A>(mut self, addr: A)
        where G: Game<GD, E>, A: ToSocketAddr {

        let (listener, _) = wire::listen_tcp(addr).unwrap();

        let (tx, rx): (mpsc::Sender<ServerClient<GD, CE, E>>,
                       mpsc::Receiver<ServerClient<GD, CE, E>>) = mpsc::channel();
        Thread::spawn(move || {
            let (read_limit, write_limit) = (SizeLimit::Infinite, SizeLimit::Infinite);

            for connection in listener.into_blocking_iter() {
                let (i, o) = wire::upgrade_tcp(connection, read_limit, write_limit);
                
                let (to_client_tx, to_client_rx) = mpsc::channel();

                // client thread
                Thread::spawn(move || {
                    let mut o = o;
                    for msg in to_client_rx.iter() {
                        o.send(&msg);
                    }
                });

                let client = ServerClient {
                    joined: false,
                    to_client: to_client_tx,
                    from_client: i
                };

                tx.send(client);
            }
        });

        let mut clients = Vec::new();

        if let Ok(mut res) = rx.recv() {
            res.to_client.send(ServerMessage::Image(self.data.clone()));
            clients.push(res);
        }

        loop {
            while let Ok(mut res) = rx.try_recv() {
                res.to_client.send(ServerMessage::Image(self.data.clone()));
                clients.push(res);
            }

            let mut events: Vec<CE> = clients.iter_mut().map(|c| {
                if c.joined {
                    let e = c.from_client.recv_block().expect("client sucks man");
                    Some(match e {
                        ClientMessage::Join => panic!("invalid message"),
                        ClientMessage::Event(e) => e
                    })
                } else {
                    match c.from_client.recv() {
                        Some(ClientMessage::Join) => c.joined = true,
                        _ => ()
                    }
                    None
                }
            }).filter(|x| x.is_some()).map(|x| x.unwrap()).collect();

            println!("events.len={}", events.len());

            if events.len() == 0 { continue }

            let event: E = events.drain().collect();

            for c in clients.iter_mut() {
                c.to_client.send(ServerMessage::Event(event.clone()));
            }
            self.data = self.game.step(event, self.data);
        }
    }
}

#[derive(Clone)]
pub struct ClientState<T, E> {
    joined: bool,
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

pub struct Client<G, T, SE, CE> {
    game: G,
    to_server: OutTcpStream<ClientMessage<CE>>,
    from_server: bchannel::Receiver<ServerMessage<T, SE>, bincode::DecodingError>
}

impl<Game, GameState, ServerEvent, ClientEvent> Client<Game, GameState, ServerEvent, ClientEvent>
    where GameState: Send+Encodable+Decodable+Clone,
          ServerEvent: Send+Encodable+Decodable+Clone,
          ClientEvent: Send+Encodable+Decodable+Clone
     {

    pub fn new<A>(game: Game, addr: A) -> Client<Game, GameState, ServerEvent, ClientEvent>
        where A: ToSocketAddr {
        let (o, i) = wire::connect_tcp(addr, SizeLimit::Infinite, SizeLimit::Infinite).unwrap();

        Client {
            game: game,
            to_server: i,
            from_server: o  
        }
    }

    pub fn gamedata(&mut self) -> ClientState<GameState, ClientEvent> {
        match self.from_server.recv_block().expect("failed to get state") {
            ServerMessage::Image(state) => {
                println!("download done");
                ClientState {
                    joined: false,
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

impl<G: Game<T, SE>,
     T:Send+Encodable+Decodable+Clone,
     SE:Send+Encodable+Decodable+Clone+FromIterator<CE>,
     CE:Send+Encodable+Decodable+Clone>
    Game<ClientState<T, CE>, CE> for Client<G, T, SE, CE> {

    fn step(&mut self, event: CE, mut gd: ClientState<T, CE>) -> ClientState<T, CE> {
        // ack the server at start
        if !gd.joined {
            self.to_server.send(&ClientMessage::Join);
            gd.joined = true;
        }

        let mut frame = gd.predict.clone();
        let mut index = gd.predict_frame;

        loop {
            match self.from_server.recv() {
                Some(ServerMessage::Image(_)) => panic!("unexpected game data"),
                Some(ServerMessage::Event(e)) => {
                    gd.server = self.game.step(e, gd.server);
                    gd.server_frame += 1;
                    frame = gd.server.clone();
                    index = gd.server_frame;
                },
                None => break
            }
        }

        self.to_server.send(&ClientMessage::Event(event.clone()));
        gd.predict_frame += 1;
        gd.predict_delta.push((gd.predict_frame, event));

        let mut predict = Vec::new();
        for (idx, e) in gd.predict_delta.drain() {
            if idx > index {
                println!("{} > {} > {}", idx, index, gd.server_frame);
                frame = self.game.step([e.clone()].iter().map(|x| x.clone()).collect(), frame);
                index = idx;
            }

            if idx > gd.server_frame {
                predict.push((idx, e.clone()));
            }
        }

        gd.predict = frame;
        gd.predict_delta = predict;
        gd.predict_frame = index;
        gd
    }
}

