
use game::Game;
use input;
use input::Event;

pub struct InputIntegrator<Game> {
    game: Game
}

impl<Game> InputIntegrator<Game> {
    fn new(game: Game) -> InputIntegrator<Game> {
        InputIntegrator { game: game }
    }
}

pub struct InputIntegratorGameData<GameData> {
    game_data: GameData
}

impl<GameData> InputIntegratorGameData<GameData> {
    fn new(game_data: GameData) -> InputIntegratorGameData<GameData> {
        InputIntegratorGameData {
            game_data: game_data
        }
    }
}

impl<T> Deref<T> for InputIntegratorGameData<T> {
    fn deref<'a>(&'a self) -> &'a T {
        &self.game_data
    }
}

impl<T> DerefMut<T> for InputIntegratorGameData<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.game_data
    }
}

fn input_integrator<Game, GameData>(game: Game, game_data: GameData) 
    -> (InputIntegrator<Game>, InputIntegratorGameData<GameData>) {

    (InputIntegrator::new(game), InputIntegratorGameData::new(game_data))
}

impl<InputGame, GameData> Game<InputIntegratorGameData<GameData>, Event> for InputIntegrator<InputGame> {
    fn step(&mut self, event: Event, gd: InputIntegratorGameData<GameData>) 
        -> InputIntegratorGameData<GameData> {

        match event {
            input::Cadance(index, time) => {
                //self.game.step()
            }
            _ => ()
        }

        gd
    }
}