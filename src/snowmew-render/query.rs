
use gl;
use gl::types::{GLuint};

pub struct Query
{
    id: GLuint
}

pub struct TimeQuery
{
    quary: Query
}

impl Query
{
    pub fn new() -> Query
    {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenQueries(1, &mut id);
        }

        Query {
            id: id
        }
    }

    pub fn start_time(self) -> TimeQuery
    {
        gl::BeginQuery(gl::TIME_ELAPSED, self.id);

        TimeQuery {
            quary: self
        }
    }
}

impl TimeQuery
{
    pub fn end(&self)
    {
        gl::EndQuery(gl::TIME_ELAPSED);
    }

    pub fn time_sync(&self) -> u64
    {
        let mut time = 0;
        unsafe {
            gl::GetQueryObjectui64v(self.quary.id, gl::QUERY_RESULT, &mut time);
        }
        time
    }

    pub fn to_query(self) -> Query
    {
        self.quary
    }
}