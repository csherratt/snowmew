//   Copyright 2014 Colin Sherratt
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

use std::ops::{Deref, DerefMut};
use snowmew::Entity;
use snowmew::common::{Common, CommonData};
use position::{Positions, PositionData};
use graphics::{Graphics, GraphicsData};
use render::{Renderable, RenderData};
use input::{GetIoState, IoState};
use debug::{DebuggerGameData};


#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct GameData {
    common: CommonData,
    position: PositionData,
    graphics: GraphicsData,
    render: RenderData,
    input: IoState,
    pub time: f64,
    pub gears: Vec<Entity>
}

impl GameData {
    pub fn new() -> GameData {
        let mut gd = GameData {
            common: CommonData::new(),
            position: PositionData::new(),
            graphics: GraphicsData::new(),
            render: RenderData::new(),
            gears: Vec::new(),
            input: IoState::new(),
            time: 0.,
        };

        gd.load_standard_graphics();
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

impl Renderable for GameData {
    fn get_render_data<'a>(&'a self) -> &'a RenderData { &self.render }
    fn get_render_data_mut<'a>(&'a mut self) -> &'a mut RenderData { &mut self.render }
}

impl GetIoState for GameData {
    fn get_io_state<'a>(&'a self) -> &'a IoState { &self.input }
    fn get_io_state_mut<'a>(&'a mut self) -> &'a mut IoState { &mut self.input }
}

#[derive(Clone)]
pub struct GearsInputData {
    pub paused: bool,
    pub inner: DebuggerGameData<GameData, f64>
}

impl Deref for GearsInputData {
    type Target = DebuggerGameData<GameData, f64>;

    fn deref<'a>(&'a self) -> &'a DebuggerGameData<GameData, f64> {
        &self.inner
    }
}

impl DerefMut for GearsInputData {
    fn deref_mut<'a>(&'a mut self) -> &'a mut DebuggerGameData<GameData, f64> {
        &mut self.inner
    }
}

impl Common for GearsInputData {
    fn get_common<'a>(&'a self) -> &'a CommonData { &self.common }
    fn get_common_mut<'a>(&'a mut self) -> &'a mut CommonData { &mut self.common }
}

impl Positions for GearsInputData {
    fn get_position<'a>(&'a self) -> &'a PositionData { &self.position }
    fn get_position_mut<'a>(&'a mut self) -> &'a mut PositionData { &mut self.position }
}

impl Graphics for GearsInputData {
    fn get_graphics<'a>(&'a self) -> &'a GraphicsData { &self.graphics }
    fn get_graphics_mut<'a>(&'a mut self) -> &'a mut GraphicsData { &mut self.graphics }
}

impl Renderable for GearsInputData {
    fn get_render_data<'a>(&'a self) -> &'a RenderData { &self.render }
    fn get_render_data_mut<'a>(&'a mut self) -> &'a mut RenderData { &mut self.render }
}

impl GetIoState for GearsInputData {
    fn get_io_state<'a>(&'a self) -> &'a IoState { &self.input }
    fn get_io_state_mut<'a>(&'a mut self) -> &'a mut IoState { &mut self.input }
}