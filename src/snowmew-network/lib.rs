

extern crate "snowmew-core" as core;
extern crate wire;
extern crate bchannel;
extern crate bincode;
extern crate "rustc-serialize" as rustc_serialize;

use std::sync::mpsc;
use std::thread::Thread;
use std::io::net::tcp::{
    TcpStream,
    TcpListener,
    TcpAcceptor
};
use std::io::{
    IoError,
};
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
                let (i, mut o) = wire::upgrade_tcp(connection, read_limit, write_limit);
                
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

pub struct Client<G, T, E> {
    game: G,
    to_server: OutTcpStream<ClientMessage<E>>,
    from_server: bchannel::Receiver<ServerMessage<T, E>, bincode::DecodingError>
}

impl<G, T:Send+Encodable+Decodable, E:Send+Encodable+Decodable> Client<G, T, E> {
    pub fn new(game: G, server: &str, port: u16) -> Client<G, T, E> {
        let (o, i) = wire::connect_tcp(server, port, SizeLimit::Infinite, SizeLimit::Infinite).unwrap();

        Client {
            game: game,
            to_server: i,
            from_server: o  
        }
    }

    pub fn gamedata(&mut self) -> T {
        match self.from_server.recv_block().expect("failed to get state") {
            ServerMessage::Image(state) => return state,
            _ => panic!("expected game data")
        }
    }
}

impl<G: Game<T, E>, T:Send+Encodable+Decodable, E:Send+Encodable+Decodable> Game<T, E> for Client<G, T, E> {
    fn step(&mut self, event: E, mut gd: T) -> T {
        self.to_server.send(&ClientMessage::Event(event));

        loop {
            match self.from_server.recv_block().expect("failed to get state") {
                ServerMessage::Image(_) => panic!("unexpected game data"),
                ServerMessage::Event(e) => {
                    gd = self.game.step(e, gd);
                }
                ServerMessage::Sync => break
            }
        }
        gd
    }
}