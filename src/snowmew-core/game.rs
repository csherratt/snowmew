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

/// The `Game` trait is used to describe what happens in a game's step.
/// It allows for none-stateful data to exist outside of the game so
/// that it can be reused at each `step` of the game without having
/// to recreate it.
///
/// a `Game` is composable like Matryoshka dolls. Each layer of the game
/// can be wrap around an inner layer which allows for reused of common
/// components. See `input_intergrator` of the `debugger` for an example.
pub trait Game<GameData, Event> {
    /// Apply an event over the GameData and produce a new copy of the
    /// GameData. This new copy represents the state of the next frame
    fn step(&mut self, event: Event, gd: GameData) -> GameData;
}