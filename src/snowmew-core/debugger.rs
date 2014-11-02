
use game::Game;
use input;
use common::{Common, CommonData};

#[deriving(Clone)]
pub struct Debugger<Game> {
    game: Game
}

impl<Game> Debugger<Game> {
    fn new(game: Game) -> Debugger<Game> {
        Debugger { game: game }
    }
}

#[deriving(Clone)]
struct StateSnapshot<GameData> {
    time: f64,
    frame: uint,
    game: GameData
}

#[deriving(Clone)]
pub struct DebuggerGameData<GameData> {
    paused: bool,
    step: bool,
    index_delta: uint,
    max_len: uint,
    time_delta: f64,
    last_time: f64,
    history: Vec<StateSnapshot<GameData>>,
    pub inner: GameData
}

impl<GameData> DebuggerGameData<GameData> {
    fn new(inner: GameData) -> DebuggerGameData<GameData> {
        DebuggerGameData {
            paused: false,
            step: false,
            index_delta: 0,
            time_delta: 0.,
            last_time: 0.,
            max_len: 16,
            history: Vec::new(),
            inner: inner
        }
    }

    fn compact(&mut self) {
        if self.history.len() < self.max_len {
            return;
        }

        let mut vec = Vec::new();
        {
            let last = self.history.iter();
            let mut this = self.history.iter().enumerate();
            this.next();

            for (last, (idx, this)) in last.zip(this) {
                vec.push((last.time - this.time, idx));
            }
        }

        vec.as_mut_slice().sort_by(|&(a, _), &(b, _)| a.partial_cmp(&b).unwrap());

        let (_, remove) = vec.pop().unwrap();
        self.history.remove(remove);
    }
}

impl<T: Common> Common for DebuggerGameData<T> {
    fn get_common<'a>(&'a self) -> &'a CommonData { self.inner.get_common() }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { self.inner.get_common_mut() }
}

pub fn debugger<Game, GameData>(game: Game, inner: GameData)
    -> (Debugger<Game>, DebuggerGameData<GameData>) {
    (Debugger::new(game), DebuggerGameData::new(inner))
}

impl<GameData: Clone,
     InputGame: Game<GameData, input::Event>>
    Game<DebuggerGameData<GameData>, input::Event> for Debugger<InputGame> {
    fn step(&mut self, event: input::Event, gd: DebuggerGameData<GameData>)
        -> DebuggerGameData<GameData> {
        let mut next = gd.clone();

        let step = !next.paused || next.step;

        let event = match event {
            input::ButtonDown(input::KeyboardF7) => {
                if let Some(frame) = next.history.pop() {
                    next.inner = frame.game;
                    next.time_delta = frame.time;
                    next.index_delta = frame.frame;
                }
                input::ButtonDown(input::KeyboardF7)
            }
            input::ButtonDown(input::KeyboardF8) => {
                next.paused = !gd.paused;
                input::ButtonDown(input::KeyboardF8)
            }
            input::ButtonDown(input::KeyboardF9) => {
                next.step = true;
                input::ButtonDown(input::KeyboardF9)
            }
            input::Cadance(_, time) => {
                if step {
                    next.time_delta += time - next.last_time;
                    next.index_delta += 1;
                }
                next.last_time = time;
                input::Cadance(next.index_delta , next.time_delta)
            }
            e => e
        };

        if step {
            if let input::Cadance(_, _) = event {
                next.history.push(StateSnapshot {
                    game: gd.inner.clone(),
                    time: next.time_delta,
                    frame: next.index_delta
                });
                next.compact();
                next.step = false;
            }

            next.inner = self.game.step(event, gd.inner);
        }
        next
    }
}