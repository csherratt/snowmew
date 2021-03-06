
extern crate "snowmew-timer" as timer;

use timer::{Phase, Timer};

#[test]
fn timer_inphase() {
    let mut timer = Timer::new(Phase::In, 0.5);
    let fired: Vec<bool> = (0..10).map(|_| timer.cycle(0.1)).collect();

    assert_eq!(&fired[],
        [true, false, false, false, true,
         false, false, false, false, true]
    );
}

#[test]
fn timer_out_of_phase() {
    let mut timer = Timer::new(Phase::OutOf, 0.5);
    let fired: Vec<bool> = (0..10).map(|_| timer.cycle(0.1)).collect();

    assert_eq!(&fired[],
        [false, false, false, false, true,
         false, false, false, false, true]
    );
}

#[test]
fn timer_average_inphase() {
    let mut timer = Timer::new(Phase::In, 1. / 24.);

    let mut cnt_fired = 0;
    let mut cnt_idle = 0;
    for _ in (0..1_000) {
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
    for _ in (0..1_000) {
        if timer.cycle(1. / 60.) {
            cnt_fired += 1;
        } else {
            cnt_idle += 1;
        }
    }

    assert_eq!(cnt_idle, 600);
    assert_eq!(cnt_fired, 400);
}

#[test]
fn timer_try_inphase() {
    let mut timer = Timer::new(Phase::In, 0.5);
    let fired: Vec<bool> = (0..5).map(|_| timer.try_cycle(0.1)).collect();

    assert_eq!(&fired[],
        [true, true, true, true, true]
    );
}

#[test]
fn timer_try_out_of_phase() {
    let mut timer = Timer::new(Phase::OutOf, 0.5);
    let fired: Vec<bool> = (0..5).map(|_| timer.try_cycle(0.1)).collect();

    assert_eq!(&fired[],
        [false, false, false, false, false]
    );
}

#[test]
fn timer_epoc_out_of_phase() {
    let mut timer = Timer::new(Phase::OutOf, 0.5);

    assert_eq!(timer.cycles_to_epoc(0.1), 5);
    assert!(timer.cycle(0.1) == false);
    assert_eq!(timer.cycles_to_epoc(0.1), 4);
    assert!(timer.cycle(0.1) == false);
    assert_eq!(timer.cycles_to_epoc(0.1), 3);
    assert!(timer.cycle(0.1) == false);
    assert_eq!(timer.cycles_to_epoc(0.1), 2);
    assert!(timer.cycle(0.1) == false);
    assert_eq!(timer.cycles_to_epoc(0.1), 1);
    assert!(timer.cycle(0.1) == true);
    assert_eq!(timer.cycles_to_epoc(0.1), 5);
    assert!(timer.cycle(0.1) == false);
}