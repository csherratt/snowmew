
use time::precise_time_s;

#[deriving(Clone)]
pub struct Timing
{
    time: ~[(f64, ~str)]
}

impl Timing
{
    pub fn new() -> Timing
    {
        Timing {
            time: ~[]
        }
    }

    pub fn reset(&mut self)
    {
        self.time = ~[(precise_time_s(), ~"start")]
    }

    pub fn mark(&mut self, name: ~str)
    {
        self.time.push((precise_time_s(), name))
    }

    pub fn dump(&self)
    {
        if self.time.len() == 0 {
            println!("no timing");
        } else {
            let (mut last, _) = self.time[0];

            for &(time, ref name) in self.time.iter() {
                print!("=={:2.1f}ms==> {:s} ", 1000. * (time - last), *name);
                last = time;
            }
            println!("");
        }
    }
}