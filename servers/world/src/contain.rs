//! Keeping a panic to what it happened in (FrameEnder fork, 2026-11-14).
//!
//! A panic anywhere in the world server ended the whole process: the release profile aborted on one, and `kawari-run`
//! didn't start the server again, so one index out of range in one player's packet (Bob's bundles 20261013-173216,
//! 20261026-093312 and five after it, 20261103-032121, 20261107-215105: four different places) dropped everyone until
//! someone restarted the container. With the profile unwinding instead:
//!
//! * a panic while a connection handles something ends that connection as a lost one does: the player is saved and
//!   the world told they left (`Contained`, used by `client_loop`);
//! * a panic in a step of the world (a message or a tick) is logged and the world goes on to the next step (`step`);
//!   the step's locks are released as it unwinds, and whatever it was half way through stays as it was;
//! * a world that panics over and over (the same broken state every tick) ends the process, and `kawari-run` starts
//!   it again fresh (`STORM`).
//!
//! Each is logged as an error with the panic's message, after the default hook has printed where it was and the
//! backtrace: every one is a bug to fix, and none is hidden by this.

use std::{
    any::Any,
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use parking_lot::Mutex;

/// What a panic said, for the log.
pub fn message(panic: &(dyn Any + Send)) -> String {
    if let Some(text) = panic.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = panic.downcast_ref::<String>() {
        text.clone()
    } else {
        "a panic that wasn't text".to_string()
    }
}

/// A future whose panic is its result instead of its task's end, so what owns it can clean up after it.
pub struct Contained<F>(Pin<Box<F>>);

impl<F: Future> Contained<F> {
    pub fn new(future: F) -> Self {
        Self(Box::pin(future))
    }
}

impl<F: Future> Future for Contained<F> {
    type Output = Result<F::Output, Box<dyn Any + Send>>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        match catch_unwind(AssertUnwindSafe(|| self.0.as_mut().poll(context))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(output)) => Poll::Ready(Ok(output)),
            Err(panic) => Poll::Ready(Err(panic)),
        }
    }
}

/// More panics than this within `STORM.1` and the world isn't getting past whatever is wrong: the process ends.
const STORM: (usize, Duration) = (20, Duration::from_secs(10));

/// When the world's steps last panicked (`step`.)
static RECENT: Mutex<Vec<Instant>> = Mutex::new(Vec::new());

/// Notes a panic now; true once there have been more than `limit.0` within `limit.1`.
fn storm(recent: &mut Vec<Instant>, now: Instant, limit: (usize, Duration)) -> bool {
    recent.retain(|at| now.duration_since(*at) < limit.1);
    recent.push(now);
    recent.len() > limit.0
}

/// Runs a step of the world (`what`: "a message", "the tick"). A panic in it is logged and the step is over; false
/// then. Ends the process when they come in a storm.
pub fn step(what: &str, body: impl FnOnce()) -> bool {
    let Err(panic) = catch_unwind(AssertUnwindSafe(body)) else {
        return true;
    };
    tracing::error!(
        "The world server panicked handling {what}: {}. That step was dropped and the world goes on; what it was \
         doing may be half done. This is a bug: please report it with the log above.",
        message(panic.as_ref())
    );
    if storm(&mut RECENT.lock(), Instant::now(), STORM) {
        tracing::error!(
            "The world server panicked more than {} times in {} s: ending it, to be started again fresh.",
            STORM.0,
            STORM.1.as_secs()
        );
        std::process::exit(101);
    }
    false
}

/// This many of a client's packets in a row unreadable, and its stream is out of step for good.
const UNREAD_PACKETS: u8 = 3;

/// Counts a client's packet that should have held `expected` segments and gave `parsed`; true once `UNREAD_PACKETS`
/// in a row gave none. One that can't be read alone (a segment Kawari doesn't know) doesn't count against a stream
/// that reads again after it.
pub fn out_of_step(unread: &mut u8, expected: u16, parsed: usize) -> bool {
    if expected > 0 && parsed == 0 {
        *unread = unread.saturating_add(1);
    } else {
        *unread = 0;
    }
    *unread >= UNREAD_PACKETS
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bob's bundle 20261110-111747: every packet after the stream lost its place failed, and the client waited.
    #[test]
    fn a_stream_out_of_step_is_dropped_and_one_bad_packet_isn_t() {
        let mut unread = 0;
        assert!(!out_of_step(&mut unread, 2, 2));
        assert!(!out_of_step(&mut unread, 1, 0));
        // It read again: not out of step.
        assert!(!out_of_step(&mut unread, 1, 1));
        assert!(!out_of_step(&mut unread, 1, 0));
        // A packet with nothing in it is neither.
        assert!(!out_of_step(&mut unread, 0, 0));
        assert!(!out_of_step(&mut unread, 3, 0));
        assert!(!out_of_step(&mut unread, 1, 0));
        assert!(out_of_step(&mut unread, 1, 0));
    }

    /// A panicking step is over, and the next one runs (the world's loop goes on.)
    #[test]
    fn a_panicking_step_is_dropped_and_the_next_runs() {
        let mut ran = 0;
        assert!(!step("a test's step", || {
            let nothing: Option<u32> = None;
            // As bitmask.rs:117 did in Bob's bundle 20261103-032121.
            let _ = [0u8; 4][nothing.map_or(8247, |n| n as usize)];
        }));
        assert!(step("a test's step", || ran += 1));
        assert_eq!(ran, 1);
    }

    /// A lock held by a step that panics is free afterwards (parking_lot's don't poison.)
    #[test]
    fn a_panicking_step_lets_go_of_its_locks() {
        let data = Mutex::new(1);
        assert!(!step("a test's step", || {
            let mut held = data.lock();
            *held = 2;
            panic!("with the lock held");
        }));
        assert_eq!(*data.try_lock().expect("the lock is free"), 2);
    }

    #[test]
    fn a_storm_is_many_panics_close_together() {
        let limit = (3, Duration::from_secs(10));
        let start = Instant::now();
        let mut recent = Vec::new();
        for n in 0..3 {
            assert!(!storm(&mut recent, start + Duration::from_secs(n), limit));
        }
        assert!(storm(&mut recent, start + Duration::from_secs(3), limit));
        // Spread out, they aren't one.
        let mut recent = Vec::new();
        for n in 0..10 {
            assert!(!storm(
                &mut recent,
                start + Duration::from_secs(n * 5),
                limit
            ));
        }
    }

    /// A connection's loop that panics gives its owner the panic, with what it borrowed still usable (the player is
    /// saved and the world told.)
    #[tokio::test]
    async fn a_future_that_panics_gives_its_panic() {
        let mut saved = vec![1];
        async fn turns(state: &mut Vec<u32>) {
            state.push(2);
            tokio::task::yield_now().await;
            panic!("index out of bounds: the len is 92 but the index is 8247");
        }
        let panic = Contained::new(turns(&mut saved)).await.unwrap_err();
        assert!(message(panic.as_ref()).contains("8247"));
        assert_eq!(saved, [1, 2]);
        assert_eq!(Contained::new(async { 7 }).await.unwrap(), 7);
    }
}
