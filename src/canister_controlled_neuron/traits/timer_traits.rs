use std::{cell::RefCell, collections::HashMap, hash::Hash, time::Duration};

use ic_cdk::api::time;
use ic_cdk_timers::{clear_timer, set_timer, set_timer_interval, TimerId};

pub trait Timer<K: Ord + Clone + Hash> {
    const NAME: &'static str;
    fn with_timer<R>(f: impl FnOnce(&RefCell<HashMap<K, TimerId>>) -> R) -> R;
}

pub trait TimerActions<T>: Timer<T>
where
    T: Ord + Clone + Hash,
{
    /// Creates a new timer with the specified duration and function.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the key associated with the timer. This key is used
    ///   to identify the timer in the internal storage.
    /// * `duration` - The duration for which the timer should run before executing
    ///   the provided function.
    /// * `func` - A closure that will be executed when the timer expires.
    ///
    /// # Returns
    ///
    /// Returns a `TimerId`, which is the identifier of the newly created timer.
    fn create_once(id: &T, duration: Duration, func: impl FnMut() + 'static) -> TimerId {
        // Clear any existing timer for this id
        Self::clear(id);
        let timer_id = set_timer(duration, func);
        Self::with_timer(|timers| {
            timers.borrow_mut().insert(id.clone(), timer_id);
        });
        timer_id
    }

    /// Creates a new recurring timer with the specified duration and function.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the key associated with the timer. This key is used
    ///   to identify the timer in the internal storage.
    /// * `duration` - The duration for which the timer should run before executing
    ///   the provided function repeatedly.
    /// * `func` - A closure that will be executed each time the timer interval elapses.
    ///
    /// # Returns
    ///
    /// Returns a `TimerId`, which is the identifier of the newly created recurring timer.
    fn create_recurring(id: &T, duration: Duration, func: impl FnMut() + 'static) -> TimerId {
        // Clear any existing timer for this id
        Self::clear(id);
        let timer_id = set_timer_interval(duration, func);
        Self::with_timer(|timers| {
            timers.borrow_mut().insert(id.clone(), timer_id);
        });
        timer_id
    }

    /// Retrieves the timer ID associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the key associated with the timer.
    ///
    /// # Returns
    ///
    /// Returns an `Option<TimerId>`, which is the identifier of the timer if it exists.
    fn get(id: &T) -> Option<TimerId> {
        Self::with_timer(|timers| timers.borrow().get(id).cloned())
    }

    /// Clears the timer associated with the given key.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the key associated with the timer.
    ///
    /// This function will remove the timer from the internal storage and cancel
    /// its execution if it is still pending.
    fn clear(id: &T) {
        if let Some(timer_id) = Self::get(id) {
            clear_timer(timer_id);
            Self::with_timer(|timers| {
                timers.borrow_mut().remove(id);
            });
        }
    }

    /// Calculates the remaining time for a timer until its end time.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the key associated with the timer.
    /// * `end_time` - The original end time of the timer in nanoseconds since the
    ///   Unix epoch.
    ///
    /// # Returns
    ///
    /// Returns an `Option<Duration>`, which is the remaining duration for the timer
    /// if it exists and the end time is in the future. Returns `None` if the timer
    /// does not exist or if the end time is in the past.
    fn get_time_left(id: &T, end_time: u64) -> Option<Duration> {
        // Check if end_time is in the future
        if end_time < time() {
            return Some(Duration::from_secs(0));
        }

        // Check if the timer exists
        Self::get(id)?;

        Some(Duration::from_nanos(end_time - time()))
    }

    /// Restores a timer after a canister upgrade.
    ///
    /// When a canister is upgraded, all active timers are cleared. This function
    /// allows you to restore a timer with the same duration and function that was
    /// previously set before the upgrade.
    ///
    /// # Arguments
    ///
    /// * `id` - A reference to the key associated with the timer. This key is used
    ///   to identify the timer that needs to be restored.
    /// * `end_time` - The original end time of the timer in nanoseconds since the
    ///   Unix epoch. This is used to calculate the remaining duration for the timer.
    /// * `func` - A closure that will be executed when the timer expires. This
    ///   function should encapsulate the logic that needs to be performed once the
    ///   timer completes.
    ///
    /// # Returns
    ///
    /// Returns an `Option<TimerId>`, which is the identifier of the newly created
    /// timer if the restoration is successful. Returns `None` if the timer could
    /// not be restored, either because it does not exist or due to other reasons.
    fn restore_timer_after_upgrade(
        id: &T,
        end_time: u64,
        is_recurring: bool,
        func: impl FnMut() + 'static,
    ) -> Option<TimerId> {
        // Check if the timer exists
        Self::get(id)?;

        // Calculate the remaining time for the timer
        let time_left = Self::get_time_left(id, end_time)?;

        // Create and set the timer with the remaining duration
        if is_recurring {
            Some(Self::create_recurring(id, time_left, func))
        } else {
            Some(Self::create_once(id, time_left, func))
        }
    }
}
