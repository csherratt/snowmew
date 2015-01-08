//   Copyright 2015 Colin Sherratt
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

#[derive(Copy, Clone)]
/// Determines when the timer should fire
pub enum Phase {
    /// The timer should fire at the start of a cycle.
    /// Think of this like a rate limiter, you can fire once then you
    /// have to wait until the next cycle to fire again.
    In,
    /// The timer should fire at the end of a cycle.
    /// this counts down until an action can happen, like charging up something
    OutOf
}

#[derive(Copy, Clone)]
pub struct Timer {
    phase: Phase,
    accumulator: u32,
    rate: f32
}

impl Timer {
    /// Create a timer that phase and rate is determined
    /// Rate is in second/cycles
    pub fn new(phase: Phase, rate: f32) -> Timer {
        Timer {
            phase: phase,
            accumulator: 0,
            rate: rate
        }
    }

    /// each timer must be cycled periodically
    /// seconds it the number of seconds that has passed
    /// This should be a fixed base
    pub fn cycle(&mut self, seconds: f32) -> bool {
        let imax = std::u32::MAX as u64 + 1;
        let max = imax as f32;
        let inc = match self.phase {
            Phase::In => {
                imax - ((seconds * max) / self.rate) as u64
            }
            Phase::OutOf => {
                ((seconds * max) / self.rate) as u64
            }
        };

        let last = self.accumulator;
        self.accumulator += inc as u32;
        
        match (self.accumulator > last, self.phase) {
            (true, Phase::In) => true,
            (false, Phase::OutOf) => true,
            _ => false
        }
    }
}
