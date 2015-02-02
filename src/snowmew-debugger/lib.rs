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

#![feature(old_impl_check)]
#![feature(alloc)]

extern crate "snowmew-core" as core;

use std::sync::Arc;
use std::collections::{VecMap, BTreeMap};
use std::ops::{Deref, DerefMut};
use core::game::Game;

#[derive(Clone)]
pub struct Debugger<Game> {
    game: Game
}

impl<Game> Debugger<Game> {
    pub fn new(game: Game) -> Debugger<Game> {
        Debugger { game: game }
    }
}

#[derive(Clone)]
pub struct DebuggerGameData<GameData, Event> {
    index: usize,
    limit: usize,
    events: Arc<VecMap<Event>>,
    history: Arc<BTreeMap<usize, GameData>>,
    pub inner: GameData
}

impl<GameData: Send+Sync+Clone, Event: Send+Sync+Clone> DebuggerGameData<GameData, Event> {
    pub fn new(inner: GameData, limit: usize) -> DebuggerGameData<GameData, Event> {
        DebuggerGameData {
            index: 0,
            limit: limit,
            events: Arc::new(VecMap::new()),
            history: Arc::new(BTreeMap::new()),
            inner: inner
        }
    }

    fn compact(&mut self) {
        while self.history.len() > self.limit {
            let mut vec = Vec::new();
            {
                let last = self.history.iter();
                let mut this = self.history.iter();
                this.next();

                for ((&last, _), (&this, _)) in last.zip(this) {
                    vec.push((this - last, this));
                }
            }

            vec.as_mut_slice().sort_by(|&(b, _), &(a, _)| a.partial_cmp(&b).unwrap());

            let (_, remove) = vec.pop().unwrap();
            self.history.make_unique().remove(&remove).is_some();
        }
    }
}

impl<E, T> Deref for DebuggerGameData<T, E> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a <Self as Deref>::Target {
        &self.inner
    }
}

impl<E, T> DerefMut for DebuggerGameData<T, E> {
    fn deref_mut <'a>(&'a mut self) -> &'a mut <Self as Deref>::Target {
        &mut self.inner
    }
}

#[old_impl_check]
impl<GameData: Clone+Send+Sync,
     Event: Clone+Send+Sync,
     InputGame: Game<GameData, Event>> Debugger<InputGame> {

    /// seek a checkpoint ahead of the current point
    pub fn skip_forward(&mut self, mut gd: DebuggerGameData<GameData, Event>)
        -> DebuggerGameData<GameData, Event> {
        let mut out = None;
        for (&idx, f) in gd.history.iter() {
            if idx > gd.index {
                out = Some((idx, f.clone()));
                break;
            }
        }
        if let Some((idx, f)) = out {
            gd.index = idx;
            gd.inner = f;
        }
        gd
    }

    /// seek a checkout before the current time
    pub fn skip_backward(&mut self, mut gd: DebuggerGameData<GameData, Event>)
        -> DebuggerGameData<GameData, Event> {
        let mut out = None;
        for (&idx, f) in gd.history.iter().rev() {
            if idx < gd.index {
                out = Some((idx, f.clone()));
                break;
            }
        }
        if let Some((idx, f)) = out {
            println!("{}", idx);
            gd.index = idx;
            gd.inner = f;
        }
        gd
    }

    pub fn replay_one(&mut self, mut gd: DebuggerGameData<GameData, Event>)
        -> DebuggerGameData<GameData, Event> {
        if let Some(event) = gd.events.get(&gd.index).map(|x| x.clone()) {
            gd.inner = self.game.step(event, gd.inner);
            gd.index += 1;
        }
        gd
    }

    pub fn revert_one(&mut self, mut gd: DebuggerGameData<GameData, Event>)
        -> DebuggerGameData<GameData, Event> {
        let idx = gd.index - 1;
        if idx == 0 {
            gd
        } else {
            gd = self.skip_backward(gd);
            while gd.index != idx {
                gd = self.replay_one(gd);
            }
            gd
        }
    }

    /// set limit of checkpoints
    pub fn limit(&mut self, limit: usize, mut gd: DebuggerGameData<GameData, Event>)
        -> DebuggerGameData<GameData, Event> {
        gd.limit = limit;
        gd.compact();
        gd
    }
}

impl<GameData: Clone+Send+Sync,
     Event: Clone+Send+Sync,
     InputGame: Game<GameData, Event>>
     Game<DebuggerGameData<GameData, Event>, Event> for Debugger<InputGame> {
    fn step(&mut self, event: Event, gd: DebuggerGameData<GameData, Event>)
        -> DebuggerGameData<GameData, Event> {

        let mut next = gd.clone();
        next.history.make_unique().insert(next.index, gd.inner.clone());
        next.events.make_unique().insert(next.index, event.clone());
        next.index += 1;
        next.compact();
        next.inner = self.game.step(event, gd.inner);
        next
    }
}
