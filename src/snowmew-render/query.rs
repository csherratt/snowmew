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

use gl;
use gl::types::{GLuint};
use time::precise_time_s;

pub struct Query {
    id: GLuint
}

pub struct TimeElapsedQuery {
    quary: Query
}

pub struct TimeStampQuery {
    quary: Query
}

impl Query {
    pub fn new() -> Query {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenQueries(1, &mut id);
        }

        Query {
            id: id
        }
    }

    pub fn start_time(self) -> TimeElapsedQuery {
        unsafe {gl::BeginQuery(gl::TIME_ELAPSED, self.id);}

        TimeElapsedQuery {
            quary: self
        }
    }

    pub fn time_stamp(self) -> TimeStampQuery {
        unsafe {gl::QueryCounter(self.id, gl::TIMESTAMP);}

        TimeStampQuery {
            quary: self
        }
    }
}

impl TimeElapsedQuery {
    pub fn end(&self) {
        unsafe { gl::EndQuery(gl::TIME_ELAPSED); }
    }

    pub fn time_sync_ns(&self) -> u64 {
        let mut time = 0;
        unsafe {
            gl::GetQueryObjectui64v(self.quary.id, gl::QUERY_RESULT, &mut time);
        }
        time
    }

    pub fn time_sync_s(&self) -> f64 {
        self.time_sync_ns() as f64 / 1_000_000_000.
    }

    pub fn to_query(self) -> Query {
        self.quary
    }
}

impl TimeStampQuery {
    pub fn time_sync_ns(&self) -> u32 {
        let mut time = 0;
        unsafe {
            gl::GetQueryObjectuiv(self.quary.id, gl::QUERY_RESULT, &mut time);
        }
        time
    }

    pub fn time_sync_s(&self) -> f64 {
        self.time_sync_ns() as f64 / 1_000_000_000.
    }

    pub fn to_query(self) -> Query {
        self.quary
    }
}

pub trait Profiler {
    fn reset(&mut self);
    fn time(&mut self, name: String);
    fn dump(&mut self);
}

pub struct TimeQueryManager {
    saved: Vec<Query>,
    log: Vec<(TimeElapsedQuery, f64, String)>,
    last: Option<(TimeElapsedQuery, f64, String)>
}

impl TimeQueryManager {
    pub fn new() -> TimeQueryManager {
        TimeQueryManager {
            saved: Vec::new(),
            log: Vec::new(),
            last: None
        }
    }

    fn get_query(&mut self) -> Query {
        match self.saved.pop() {
            Some(q) => q,
            None => Query::new()
        }
    }

    fn done(&mut self) {
        match self.last.take() {
            Some((q, start, name)) => {
                q.end();
                self.log.push((q, start, name))
            }
            None => ()
        };        
    }
}

impl Profiler for TimeQueryManager {
    fn reset(&mut self) {
        while !self.log.is_empty() {
            let (query, _, _) = self.log.pop().unwrap();
            self.saved.push(query.to_query());
        }
    }

    fn time(&mut self, name: String) {
        self.done();
        let start = precise_time_s();
        let query = self.get_query().start_time();
        self.last = Some((query, start, name));
    }

    fn dump(&mut self) {
        self.time("done".to_string());
        self.done();

        let len = self.log.len();
        for (&(ref query, start, ref name), &(_, end, _)) 
                in self.log.slice(0, len-1).iter().zip(self.log.slice(1, len).iter()) {
            let cpu_time = end - start;
            let gpu_time = query.time_sync_s();
            println!("{:30s} | {:6.2f}ms | {:6.2f}ms", *name, cpu_time * 1000., gpu_time * 1000.);
        }
    }
}

pub struct ProfilerDummy;

impl Profiler for ProfilerDummy {
    fn reset(&mut self) {}
    fn time(&mut self, _: String) {}
    fn dump(&mut self) {}
}