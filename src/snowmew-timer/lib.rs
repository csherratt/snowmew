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

#![feature(core)]

extern crate "rustc-serialize" as rustc_serialize;

use std::intrinsics::overflowing_add;

#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
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

#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
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

    /// calculate the increment
    fn increment(&self, seconds: f32) -> u32 {
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
        inc as u32
    }

    /// each timer must be cycled periodically
    /// seconds it the number of seconds that has passed
    /// This should be a fixed base
    pub fn cycle(&mut self, seconds: f32) -> bool {
        let last = self.accumulator;
        self.accumulator = unsafe { overflowing_add(self.accumulator, self.increment(seconds)) };
        
        match (self.accumulator > last, self.phase) {
            (true, Phase::In) => true,
            (false, Phase::OutOf) => true,
            _ => false
        }
    }

    /// check to see if calling cycle() will start a new
    /// cycle, iff it will zero the accumulator, iff it doen't
    /// this is the equivalent of calling cycle()
    pub fn try_cycle(&mut self, seconds: f32) -> bool {
        let last = self.accumulator;
        let next = self.accumulator + self.increment(seconds);
        
        match (next > last, self.phase) {
            (true, Phase::In) => {
                self.accumulator = 0;
                true
            }
            (false, Phase::In) => {
                self.accumulator = next;
                false
            }
            (x, Phase::OutOf) => {
                self.accumulator = 0;
                !x
            }
        }
    }

    /// calculate the number of cycles to epoc
    pub fn cycles_to_epoc(&self, seconds: f32) -> u32 {
        let amount = self.accumulator as f64 / (std::u32::MAX as f64 + 1.);
        let total_cycles = self.rate as f64 / seconds as f64;
        return ((1. - amount) * total_cycles) as u32 + 1;
    }

    /// calculate the number cycles in a timer cycle
    pub fn total_cycles(&self, seconds: f32) -> u32 {
        return (self.rate as f64 / seconds as f64) as u32;
    }

    /// calculate the % of cycles until a timer is done
    pub fn percent_done(&self, seconds: f32) -> f32 {
        let total = self.total_cycles(seconds) as f32;
        let to_epoc = self.cycles_to_epoc(seconds) as f32;
        return (total - to_epoc) / total;
    }

    /// sets the rate to a new value
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
    }
}
