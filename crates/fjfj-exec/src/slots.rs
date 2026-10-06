//! The `--jobs` slots, handed to the waiting action with the most work still
//! behind it rather than to the one that has waited longest (buildfiji-98c9).
//!
//! Bazel starts what is on the critical path first. When slots are scarce a
//! first-in first-out queue lets a short action that many others wait for sit
//! behind a pile of compiles that nothing waits for.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

pub struct Slots {
    state: Mutex<State>,
}

struct State {
    free: usize,
    next: u64,
    waiting: BinaryHeap<Waiter>,
}

/// Ordered by rank, then by arrival: the greater is served first.
struct Waiter {
    rank: u64,
    arrival: Reverse<u64>,
    wake: oneshot::Sender<()>,
}

impl PartialEq for Waiter {
    fn eq(&self, other: &Self) -> bool {
        (self.rank, self.arrival) == (other.rank, other.arrival)
    }
}
impl Eq for Waiter {}
impl PartialOrd for Waiter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Waiter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.rank, self.arrival).cmp(&(other.rank, other.arrival))
    }
}

/// A slot; dropping it hands the slot on.
pub struct Slot {
    slots: Arc<Slots>,
}

impl Slots {
    pub fn new(count: usize) -> Arc<Self> {
        Arc::new(Slots {
            state: Mutex::new(State {
                free: count,
                next: 0,
                waiting: BinaryHeap::new(),
            }),
        })
    }

    /// A slot, once one is free and no waiting action has a greater `rank`.
    pub async fn acquire(self: &Arc<Self>, rank: u64) -> Slot {
        let wait = {
            let mut state = self.state.lock().unwrap();
            if state.free > 0 && state.waiting.is_empty() {
                state.free -= 1;
                None
            } else {
                let (wake, woken) = oneshot::channel();
                let arrival = Reverse(state.next);
                state.next += 1;
                state.waiting.push(Waiter {
                    rank,
                    arrival,
                    wake,
                });
                Some(woken)
            }
        };
        if let Some(woken) = wait {
            // A slot is passed to a waiter directly, so there is nothing to retake.
            let _ = woken.await;
        }
        Slot {
            slots: self.clone(),
        }
    }

    fn release(&self) {
        let mut state = self.state.lock().unwrap();
        // A waiter whose future was dropped cannot take the slot.
        while let Some(next) = state.waiting.pop() {
            if next.wake.send(()).is_ok() {
                return;
            }
        }
        state.free += 1;
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        self.slots.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_waiting_action_with_the_most_work_behind_it_takes_a_freed_slot() {
        let slots = Slots::new(1);
        let held = slots.acquire(0).await;
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut tasks = Vec::new();
        // Queued first, but little behind it.
        for (name, rank) in [("short", 1), ("long", 100), ("middle", 10)] {
            let (slots, order) = (slots.clone(), order.clone());
            tasks.push(tokio::spawn(async move {
                let _slot = slots.acquire(rank).await;
                order.lock().unwrap().push(name);
            }));
            tokio::task::yield_now().await;
        }
        drop(held);
        for task in tasks {
            task.await.unwrap();
        }
        assert_eq!(*order.lock().unwrap(), ["long", "middle", "short"]);
    }

    #[tokio::test]
    async fn equal_ranks_are_served_in_arrival_order() {
        let slots = Slots::new(1);
        let held = slots.acquire(0).await;
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut tasks = Vec::new();
        for name in ["a", "b", "c"] {
            let (slots, order) = (slots.clone(), order.clone());
            tasks.push(tokio::spawn(async move {
                let _slot = slots.acquire(5).await;
                order.lock().unwrap().push(name);
            }));
            tokio::task::yield_now().await;
        }
        drop(held);
        for task in tasks {
            task.await.unwrap();
        }
        assert_eq!(*order.lock().unwrap(), ["a", "b", "c"]);
    }

    #[tokio::test]
    async fn a_waiter_that_gives_up_does_not_keep_a_slot() {
        let slots = Slots::new(1);
        let held = slots.acquire(0).await;
        let gone = tokio::spawn({
            let slots = slots.clone();
            async move {
                let _ = slots.acquire(50).await;
            }
        });
        tokio::task::yield_now().await;
        gone.abort();
        let _ = gone.await;
        drop(held);
        // The slot is free again for the next action.
        let _again = tokio::time::timeout(std::time::Duration::from_secs(5), slots.acquire(1))
            .await
            .expect("the slot is free");
    }
}
