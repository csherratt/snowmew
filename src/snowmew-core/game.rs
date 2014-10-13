

pub trait Game<GameData, Edge> {
    fn step(&mut self, edge: Edge, gd: GameData) -> GameData;
}
