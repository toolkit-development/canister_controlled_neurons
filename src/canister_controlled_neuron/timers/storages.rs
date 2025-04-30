use ic_cdk_timers::TimerId;
use std::{cell::RefCell, collections::HashMap};

use crate::traits::timer_traits::{Timer, TimerActions};

thread_local! {
    pub static TIMERS: RefCell<HashMap<[u8; 32], TimerId>> = RefCell::new(HashMap::default());
    pub static COUNTER: RefCell<u64> = const { RefCell::new(0) };
}

pub struct Timers;

impl Timer<[u8; 32]> for Timers {
    const NAME: &'static str = "timers";

    fn with_timer<R>(f: impl FnOnce(&RefCell<HashMap<[u8; 32], TimerId>>) -> R) -> R {
        TIMERS.with(f)
    }
}

impl TimerActions<[u8; 32]> for Timers {}
