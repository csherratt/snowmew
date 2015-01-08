
extern crate "snowmew-timer" as timer;

use timer::{Phase, Timer};

#[test]
fn timer_inphase() {
    let mut timer = Timer::new(Phase::In, 0.5);
    let fired: Vec<bool> = range(0, 10).map(|_| timer.cycle(0.1)).collect();

    assert_eq!(fired.as_slice(),
        [true, false, false, false, true,
         false, false, false, false, true]
    );
}

#[test]
fn timer_out_of_phase() {
    let mut timer = Timer::new(Phase::OutOf, 0.5);
    let fired: Vec<bool> = range(0, 10).map(|_| timer.cycle(0.1)).collect();

    assert_eq!(fired.as_slice(),
        [false, false, false, false, true,
         false, false, false, false, true]
    );
}

#[test]
fn timer_average_inphase() {
    let mut timer = Timer::new(Phase::In, 1. / 24.);

    let mut cnt_fired = 0;
    let mut cnt_idle = 0;
    for _ in range(0, 1_000) {
        if timer.cycle(1. / 60.) {
            cnt_fired += 1;
        } else {
            cnt_idle += 1;
        }
    }

    assert_eq!(cnt_idle, 599);
    assert_eq!(cnt_fired, 401);
}


#[test]
fn timer_average_out_of_phase() {
    let mut timer = Timer::new(Phase::OutOf, 1. / 24.);

    let mut cnt_fired = 0;
    let mut cnt_idle = 0;
    for _ in range(0, 1_000) {
        if timer.cycle(1. / 60.) {
            cnt_fired += 1;
        } else {
            cnt_idle += 1;
        }
    }

    assert_eq!(cnt_idle, 600);
    assert_eq!(cnt_fired, 400);
}
