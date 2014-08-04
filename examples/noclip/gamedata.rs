
use snowmew::common::{Common, CommonData};
use position::{Positions, PositionData};
use graphics::{Graphics, GraphicsData};
use graphics::default::load_default;
use render_data::RenderData;

#[deriving(Clone)]
pub struct GameData {
    common: CommonData,
    position: PositionData,
    graphics: GraphicsData
}

impl GameData {
    pub fn new() -> GameData {
        let mut gd = GameData {
            common: CommonData::new(),
            position: PositionData::new(),
            graphics: GraphicsData::new()
        };

        load_default(&mut gd);

        gd
    }
}

impl Common for GameData {
    fn get_common<'a>(&'a self) -> &'a CommonData { &self.common }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { &mut self.common }
}

impl Positions for GameData {
    fn get_position<'a>(&'a self) -> &'a PositionData { &self.position }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { &mut self.position }
}

impl Graphics for GameData {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData { &self.graphics }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData { &mut self.graphics }
}

impl RenderData for GameData {}