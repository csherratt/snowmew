

pub trait Game<GameData, Event> {
    fn step(&mut self, event: Event, gd: GameData) -> GameData;
}