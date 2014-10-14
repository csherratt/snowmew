
use game::Game;
use input;
use input_integrator::InputIntegratorState;
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
pub struct DebuggerGameData<GameData> {
    paused: bool,
    step: bool,
    pub inner: GameData
}

impl<GameData> DebuggerGameData<GameData> {
    fn new(inner: GameData) -> DebuggerGameData<GameData> {
        DebuggerGameData {
            paused: false,
            step: false,
            inner: inner
        }
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
     InputGame: Game<GameData, InputIntegratorState>>
    Game<DebuggerGameData<GameData>, InputIntegratorState> for Debugger<InputGame> {
    fn step(&mut self, state: InputIntegratorState, gd: DebuggerGameData<GameData>) 
        -> DebuggerGameData<GameData> {
        let mut next = gd.clone();

        if state.button_pressed(input::KeyboardF8) {
            next.paused = !gd.paused;
        }

        if next.paused && state.button_pressed(input::KeyboardF9) {
            next.step = true;
        }

        if !next.paused || next.step {
            next.inner = self.game.step(state, gd.inner);
            next.step = false;
        }
        next
    }
}