
use game::Game;
use input;
use input::Event;
use input::Button;
use common::{Common, CommonData};
use std::collections::HashMap;

#[deriving(Clone)]
pub struct InputIntegrator<Game> {
    game: Game
}

impl<Game> InputIntegrator<Game> {
    fn new(game: Game) -> InputIntegrator<Game> {
        InputIntegrator { game: game }
    }
}

#[deriving(Clone)]
pub struct InputIntegratorState {
    buttons_down: HashMap<Button, uint>,
    index: uint,
    time: f64,
    last_mouse: Option<(uint, f64, f64)>,
    mouse: Option<(uint, f64, f64)>
}

impl InputIntegratorState {
    pub fn button_down(&self, button: Button) -> bool {
        self.buttons_down.find(&button).is_some()
    }

    pub fn mouse_position(&self) -> (f64, f64) {
        if let Some((_, x, y)) = self.mouse {
            (x, y)
        } else {
            (0., 0.)
        }
    }

    pub fn mouse_delta(&self) -> (f64, f64) {
        match (self.mouse, self.last_mouse) {
            (Some((new_i, new_x, new_y)),
             Some((old_i, old_x, old_y))) if old_i != new_i => {
                (new_x - old_x, new_y - old_y)
            }
            (Some((new_i, _, _)),
             Some((old_i, _, _))) if old_i == new_i => (0., 0.),
            (Some((_, x, y)), _) => (x, y),
            _ => (0., 0.)
        }
    }
}

#[deriving(Clone)]
pub struct InputIntegratorGameData<GameData> {
    state: InputIntegratorState,
    pub inner: GameData
}

impl<GameData> InputIntegratorGameData<GameData> {
    fn new(inner: GameData) -> InputIntegratorGameData<GameData> {
        InputIntegratorGameData {
            state: InputIntegratorState {
                buttons_down: HashMap::new(),
                index: 0,
                time: 0.,
                last_mouse: None,
                mouse: None
            },
            inner: inner
        }
    }
}

impl<T: Common> Common for InputIntegratorGameData<T> {
    fn get_common<'a>(&'a self) -> &'a CommonData { self.inner.get_common() }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { self.inner.get_common_mut() }
}

pub fn input_integrator<Game, GameData>(game: Game, inner: GameData) 
    -> (InputIntegrator<Game>, InputIntegratorGameData<GameData>) {
    (InputIntegrator::new(game), InputIntegratorGameData::new(inner))
}

impl<GameData, InputGame: Game<GameData, InputIntegratorState>>
    Game<InputIntegratorGameData<GameData>, Event> for InputIntegrator<InputGame> {
    fn step(&mut self, event: Event, gd: InputIntegratorGameData<GameData>) 
        -> InputIntegratorGameData<GameData> {
        let mut gd = gd;

        match event {
            input::Cadance(index, time) => {
                gd.state.index = index;
                gd.state.time = time;
                gd.inner = self.game.step(gd.state.clone(), gd.inner);
                gd.state.last_mouse = gd.state.mouse;
            }
            input::ButtonDown(button) => {
                gd.state.buttons_down.insert(button, gd.state.index);
            }
            input::ButtonUp(button) => {
                gd.state.buttons_down.remove(&button);
            }
            input::Move(x, y) => {
                gd.state.mouse = Some((gd.state.index, x, y));
            }
        }

        gd
    }
}