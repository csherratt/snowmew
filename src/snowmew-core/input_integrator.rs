
use game::Game;
use input;
use input::Event;
use input::Button;
use common::{Common, CommonData};
use std::collections::{HashSet, HashMap};

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
    buttons_released: HashSet<Button>,
    index: uint,
    time: f64,
    last_mouse: Option<(uint, f64, f64)>,
    mouse: Option<(uint, f64, f64)>,
    scroll: (int, int),
    last_scroll: (int, int)
}

impl InputIntegratorState {
    pub fn button_down(&self, button: Button) -> bool {
        self.buttons_down.find(&button).is_some()
    }

    pub fn button_pressed(&self, button: Button) -> bool {
        if let Some(&x) = self.buttons_down.find(&button) {
            self.index - x == 1
        } else {
            false
        }
    }

    pub fn button_released(&self, button: Button) -> bool {
        self.buttons_released.contains(&button)
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

    pub fn scroll_position(&self) -> (int, int) { self.scroll }

    pub fn scroll_delta(&self) -> (int, int) {
        let (x, y) = self.scroll;
        let (ox, oy) = self.last_scroll;
        (x - ox, y - oy)
    }

    pub fn index(&self) -> uint { self.index }
    pub fn time(&self) -> f64 { self.time }
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
                buttons_released: HashSet::new(),
                index: 0,
                time: 0.,
                last_mouse: None,
                mouse: None,
                scroll: (0, 0),
                last_scroll: (0, 0)
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

impl<GameData,
     InputGame: Game<GameData, InputIntegratorState>>
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
                gd.state.buttons_released.clear();
                gd.state.last_scroll = gd.state.scroll;
            }
            input::ButtonDown(button) => {
                gd.state.buttons_down.insert(button, gd.state.index);
            }
            input::ButtonUp(button) => {
                gd.state.buttons_down.remove(&button);
                gd.state.buttons_released.insert(button);
            }
            input::Move(x, y) => {
                gd.state.mouse = Some((gd.state.index, x, y));
            }
            input::Scroll(dx, dy) => {
                let (x, y) = gd.state.scroll;
                gd.state.scroll = (x + dx, y + dy);
            }
        }

        gd
    }
}