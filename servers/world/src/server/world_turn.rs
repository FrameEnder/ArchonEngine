//! The world's turn: the one thing the world server's two tasks take before they touch its state.
//!
//! Two tasks change the world: the message loop (`server_main_loop`'s `while let Some(msg)`, one client message at a
//! time) and the tick (`server_logic_tick` every `SERVER_TICK_MILLISECONDS`, and the queued tasks every
//! `TASK_TICK_MILLISECONDS`). Both lock the world's mutexes (`data`, `network`, `game_data`, `lua`), and not in one
//! order: the tick takes `data` then `network`, while about 45 message handlers take `network` then `data` (and about
//! 75 places the other way round). When one of them came in while the tick was starting, each held the lock the
//! other was waiting for, and neither ever let go: Bob's bundle 20261105-160751, where an idle character's pose
//! change (ClientTrigger ReapplyPose, `network` then `data`) arrived at 15:22:30.337 and the tick was due at .338.
//! From then on nothing in the world moved (no NPC moves, no rested EXP), nobody new could log in (their zone
//! connection opened and the world never answered it), and the connections still answered pings from their own
//! tasks, so the client only gave up 20 minutes later and nothing was logged.
//!
//! Each task now holds the turn for a whole step (a message, or a tick with its queued tasks), so they never hold
//! the world's locks at the same time, whatever order a step takes them in. Neither step awaits anything while it
//! holds the turn.
//!
//! The watchdog is how a step that never ends shows: a thread that, when a step has held the turn longer than
//! `STUCK_AFTER`, logs an error saying which step and since when (once per stall), and logs again when it ends.

use parking_lot::{Mutex, MutexGuard};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// How long a step may hold the turn before the watchdog says the world is stuck. A tick takes milliseconds; loading
/// a zone's navmesh and collision a few seconds.
pub const STUCK_AFTER: Duration = Duration::from_secs(15);
/// A step held this long won't end: the server ends itself to be started again (`watch`.)
const GIVE_UP_AFTER: Duration = Duration::from_secs(180);
/// How often the watchdog looks.
const WATCH_EVERY: Duration = Duration::from_secs(5);

/// What holds the turn: the step and when it started.
#[derive(Default)]
struct Holder {
    /// Unix time in ms the step started, 0 while no one holds the turn.
    since: AtomicU64,
    /// Which step (`STEPS`'s index.)
    step: AtomicU64,
}

/// The steps, by what the watchdog calls them.
const STEPS: [&str; 2] = ["a client message", "the tick"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Message = 0,
    Tick = 1,
}

/// The turn, shared by the message loop and the tick.
#[derive(Clone, Default)]
pub struct WorldTurn {
    lock: Arc<Mutex<()>>,
    holder: Arc<Holder>,
}

/// Held for a step; letting it go ends the step.
pub struct Turn<'a> {
    _guard: MutexGuard<'a, ()>,
    holder: &'a Holder,
}

impl Drop for Turn<'_> {
    fn drop(&mut self) {
        self.holder.since.store(0, Ordering::Release);
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl WorldTurn {
    /// Waits for the turn, then holds it for `step` until what it returns is dropped.
    pub fn take(&self, step: Step) -> Turn<'_> {
        let guard = self.lock.lock();
        self.holder.step.store(step as u64, Ordering::Release);
        self.holder.since.store(now_ms(), Ordering::Release);
        Turn {
            _guard: guard,
            holder: &self.holder,
        }
    }

    /// Which step holds the turn and for how long, if one has for longer than `after`.
    pub fn stuck(&self, after: Duration, now: u64) -> Option<(&'static str, u64, Duration)> {
        let since = self.holder.since.load(Ordering::Acquire);
        if since == 0 || now < since {
            return None;
        }
        let held = Duration::from_millis(now - since);
        (held >= after).then(|| {
            let step = self.holder.step.load(Ordering::Acquire) as usize;
            (STEPS.get(step).copied().unwrap_or("a step"), since, held)
        })
    }

    /// Starts the watchdog thread (see the module's comment.)
    pub fn watch(&self) {
        let turn = self.clone();
        let started = std::thread::Builder::new()
            .name("world-watchdog".into())
            .spawn(move || {
                // The stall being reported (its start), so each is logged once.
                let mut reported: Option<u64> = None;
                loop {
                    std::thread::sleep(WATCH_EVERY);
                    let stuck = turn.stuck(STUCK_AFTER, now_ms());
                    if let Some((step, since, held)) = stuck
                        && held >= GIVE_UP_AFTER
                    {
                        // A step that long isn't going to end (a loop that never finishes): nothing moves and no one
                        // can log in until a restart, so it restarts itself (`kawari-run` starts the server again).
                        // Characters were saved by their connections within the last 30 s (the autosave.)
                        tracing::error!(
                            "The world server has been stuck in {step} for {} s (since {} UTC): ending it, to be \
                             started again.",
                            held.as_secs(),
                            clock(since)
                        );
                        std::process::exit(102);
                    }
                    match stuck {
                        Some((step, since, held)) if reported != Some(since) => {
                            tracing::error!(
                                "The world server is stuck: {step} has held it for {} s (since {} UTC). Nothing in \
                                 the world moves and no one can log in until it ends; this needs a restart and a bug \
                                 report with the log around that time.",
                                held.as_secs(),
                                clock(since)
                            );
                            reported = Some(since);
                        }
                        Some(_) => {}
                        None => {
                            if let Some(since) = reported.take() {
                                tracing::warn!(
                                    "The world server's step that started at {} UTC has ended.",
                                    clock(since)
                                );
                            }
                        }
                    }
                }
            });
        if let Err(err) = started {
            tracing::warn!("Couldn't start the world server's watchdog: {err}");
        }
    }
}

/// A Unix time in ms as HH:MM:SS.mmm (UTC.)
fn clock(ms: u64) -> String {
    let day = ms % 86_400_000;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        day / 3_600_000,
        day / 60_000 % 60,
        day / 1000 % 60,
        day % 1000
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two orders that deadlocked the world (the tick's `data` then `network`, a pose change's `network` then
    /// `data`), taken by two threads at once many times over: with the turn taken first, they always finish.
    #[test]
    fn opposite_orders_finish_with_the_turn() {
        let turn = WorldTurn::default();
        let data = Arc::new(Mutex::new(0u32));
        let network = Arc::new(Mutex::new(0u32));
        let tick = {
            let (turn, data, network) = (turn.clone(), data.clone(), network.clone());
            std::thread::spawn(move || {
                for _ in 0..20_000 {
                    let _turn = turn.take(Step::Tick);
                    let mut data = data.lock();
                    let mut network = network.lock();
                    *data += 1;
                    *network += 1;
                }
            })
        };
        let messages = {
            let (turn, data, network) = (turn.clone(), data.clone(), network.clone());
            std::thread::spawn(move || {
                for _ in 0..20_000 {
                    let _turn = turn.take(Step::Message);
                    let mut network = network.lock();
                    let mut data = data.lock();
                    *network += 1;
                    *data += 1;
                }
            })
        };
        tick.join().unwrap();
        messages.join().unwrap();
        assert_eq!(*data.lock(), 40_000);
        assert_eq!(*network.lock(), 40_000);
    }

    #[test]
    fn stuck_says_which_step_and_since_when() {
        let turn = WorldTurn::default();
        assert!(turn.stuck(STUCK_AFTER, now_ms()).is_none());
        {
            let _held = turn.take(Step::Tick);
            let since = turn.holder.since.load(Ordering::Acquire);
            assert!(turn.stuck(STUCK_AFTER, since + 1000).is_none());
            assert_eq!(
                turn.stuck(STUCK_AFTER, since + 20_000),
                Some(("the tick", since, Duration::from_secs(20)))
            );
        }
        // Let go: not stuck any more.
        assert!(turn.stuck(STUCK_AFTER, now_ms() + 60_000).is_none());
    }

    #[test]
    fn clock_reads_utc() {
        // 2026-11-05 15:22:30.337 UTC, the stall of bundle 20261105-160751.
        assert_eq!(clock(1_793_892_150_337), "15:22:30.337");
    }
}
