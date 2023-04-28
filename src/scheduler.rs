use crate::keys::Key;

use std::cell::Cell;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct EventEntry {
    time: Reverse<Duration>,
    entity_key: Key,
}

impl EventEntry {
    pub(crate) fn new(time: Duration, entity_key: Key) -> Self {
        Self {
            time: Reverse(time),
            entity_key,
        }
    }
    pub fn key(&self) -> Key {
        self.entity_key
    }
}

impl PartialEq for EventEntry {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for EventEntry {}

impl PartialOrd for EventEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Ord for EventEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

type Clock = Rc<Cell<Duration>>;

pub struct ClockRef {
    clock: Clock,
}

impl From<Clock> for ClockRef {
    fn from(clock: Clock) -> Self {
        Self { clock }
    }
}

impl ClockRef {
    /// Return the current simulation time.
    #[must_use]
    pub fn time(&self) -> Duration {
        self.clock.get()
    }
}

#[derive(Debug)]
pub struct Scheduler {
    pub(crate) events: BinaryHeap<EventEntry>,
    clock: Clock,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            events: BinaryHeap::default(),
            clock: Rc::new(Cell::new(Duration::ZERO)),
        }
    }
}

impl Scheduler {
    /// Schedules `event` to be executed for `entity` at `self.time() + time`.
    ///
    /// `entity_key` is a [`Key`](crate::keys::Key) corresponding to the [Generator](crate::GenBoxed) to be scheduled.
    /// 
    /// If `entity_key` was already scheduled it will ignore the following calls
    pub fn schedule(&mut self, time: Duration, entity_key: Key) {
        let already_inserted = self.events.iter().any(|ev_entry| ev_entry.entity_key == entity_key);
        if already_inserted {
            return;
        }
        let time = self.time() + time;
        let event = EventEntry::new(time, entity_key);
        self.events.push(event);
    }

    /// Schedules `event` to be executed for `entity` at `self.time()`.
    ///
    /// `entity` is a [`Key`](crate::key::Key) corresponding to the [Generator](crate::GenBoxed) to be scheduled.
    /// 
    /// If `entity_key` was already scheduled it will ignore the following calls
    pub fn schedule_now(&mut self, entity: Key) {
        self.schedule(Duration::ZERO, entity);
    }

    /// Returns the current simulation time.
    #[must_use]
    pub fn time(&self) -> Duration {
        self.clock.get()
    }

    /// Returns a structure with immutable access to the simulation time.
    #[must_use]
    pub fn clock(&self) -> ClockRef {
        ClockRef {
            clock: Rc::clone(&self.clock),
        }
    }

    /// Removes and returns the next scheduled event or `None` if none are left.
    pub fn pop(&mut self) -> Option<EventEntry> {
        self.events.pop().map(|event| {
            self.clock.replace(event.time.0);
            event
        })
    }

    pub fn remove(&mut self, key: Key) -> bool {
        if !self.events.iter().any(|event_entry| event_entry.key() == key) { return false };
        let mut events = std::mem::take(&mut self.events).into_vec();
        events.retain(|event_entry| event_entry.key() != key);
        let events = BinaryHeap::from(events);
        self.events = events;
        true
    }

    // Private function to insert `EventEntry` for testing.
    // Not used in public API
    #[allow(dead_code)]
    fn insert(&mut self, event: EventEntry) {
        // let next = self.get_new_id();
        self.events.push(event);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clock_ref_update() {
        let mut time = Duration::from_secs(1);
        let clock = Clock::new(Cell::new(time));
        let clock_ref = ClockRef::from(clock.clone());
        assert_eq!(clock_ref.time(), time);
        time += Duration::from_secs(5);
        clock.set(time);
        assert_eq!(clock_ref.time(), time);
    }

    #[test]
    fn event_entry_cmp() {
        assert_eq!(
            EventEntry {
                time: Reverse(Duration::from_secs(1)),
                entity_key: Key::new(2)
            },
            EventEntry {
                time: Reverse(Duration::from_secs(1)),
                entity_key: Key::new(2)
            }
        );
        assert_eq!(
            EventEntry {
                time: Reverse(Duration::from_secs(0)),
                entity_key: Key::new(2)
            }
            .cmp(&EventEntry {
                time: Reverse(Duration::from_secs(1)),
                entity_key: Key::new(2)
            }),
            Ordering::Greater
        );
        assert_eq!(
            EventEntry {
                time: Reverse(Duration::from_secs(2)),
                entity_key: Key::new(2)
            }
            .cmp(&EventEntry {
                time: Reverse(Duration::from_secs(1)),
                entity_key: Key::new(2)
            }),
            Ordering::Less
        );
    }

    // #[test]
    // fn scheduler_and_event_entry() {
    //     let mut scheduler = Scheduler::default();
    //     let clock_ref = scheduler.clock();
    //     let mut key_id = 0;
    //     let mut make_event_entry = |x: u64| -> EventEntry {
    //         key_id += 1;
    //         EventEntry {
    //             time: Reverse(Duration::from_secs(x) + clock.time()),
    //             entity_key: Key::new(key_id),
    //         }
    //     };
    //     let event_1 = make_event_entry(1); // Output order:
    //     let event_2 = make_event_entry(8); // event_1 -> event_3 -> event_2;
    //     let event_3 = make_event_entry(4); // Simulation Time after executing these 3 events: 8 sec.

    //     let (c_event_1, c_event_2, c_event_3) = (event_1.clone(), event_2.clone(), event_3.clone());
    //     scheduler.insert(event_1);
    //     scheduler.insert(event_2);
    //     scheduler.insert(event_3);

    //     assert_eq!(Duration::ZERO, scheduler.time()); // Assert that inserting events will not advance the simulation time.

    //     let r_event = scheduler.pop(); // Extract the event closer to the actual simulation time.
    //     assert_eq!(Some(c_event_1), r_event); // Assert that the extracted event is event_1.
    //     assert_eq!(Duration::from_secs(1), scheduler.time()); // The simulation time advance to when the event was scheduled.
    //                                                           //
    //     let r_event = scheduler.pop(); // Do the same for the other events.
    //     assert_eq!(Some(c_event_3), r_event);
    //     assert_eq!(Duration::from_secs(4), scheduler.time());

    //     let r_event = scheduler.pop();
    //     assert_eq!(Duration::from_secs(8), scheduler.time());
    //     assert_eq!(Some(c_event_2), r_event);

    //     let r_event = scheduler.pop();
    //     assert_eq!(None, r_event); // All events were extracted no more events remains in the Scheduler.
    //     assert_eq!(Duration::from_secs(8), scheduler.time()); // Actual Simulation Time: 8 sec.

    //     let event_4 = make_event_entry(10); // Schedule in Simulation Time + 10 sec.
    //     let event_5 = make_event_entry(2); // Schedule in Simulation Time + 2 seg.
    //     let (c_event_4, c_event_5) = (event_4.clone(), event_5.clone());

    //     scheduler.insert(event_4); // Output order: event_5 -> event_4
    //     scheduler.insert(event_5); // Simulation Time after extracting these 2 events: 18 sec.
    //                                //
    //     let r_event = scheduler.pop(); // Extract the inserted events
    //     assert_eq!(Some(c_event_5), r_event); // The closer one is extracted first no mather if it was inserted later.
    //     assert_eq!(Duration::from_secs(10), scheduler.time()); // The simulation time is replaced by Simulation Time + Event Time
    //                                                            // i.e Simulation Time = 8 secs + 2 secs;
    //     let r_event = scheduler.pop();
    //     assert_eq!(Some(c_event_4), r_event);
    //     assert_eq!(Duration::from_secs(18), scheduler.time());
    // }

    #[test]
    fn scheduler_and_event_entry() {
        let mut scheduler = Scheduler::default();
        let clock_ref = scheduler.clock();
        let mut key_id = 0;
        let mut make_event_entry = |x: u64| -> EventEntry {
            key_id += 1;
            EventEntry {
                time: Reverse(Duration::from_secs(x) + clock_ref.time()),
                entity_key: Key::new(key_id),
            }
        };
        let event_1 = make_event_entry(4); 
        let event_2 = make_event_entry(1);

        let (c_event_1, c_event_2) = (event_1.clone(), event_2.clone());
        scheduler.insert(event_1);
        scheduler.insert(event_2);

        assert_eq!(Duration::ZERO, scheduler.time());

        let r_event = scheduler.pop();
        assert_eq!(Some(c_event_2), r_event);
        assert_eq!(Duration::from_secs(1), scheduler.time());

        let r_event = scheduler.pop();
        assert_eq!(Some(c_event_1), r_event);
        assert_eq!(Duration::from_secs(4), scheduler.time());

        let r_event = scheduler.pop();
        assert_eq!(None, r_event); 
        assert_eq!(Duration::from_secs(4), scheduler.time()); 
    }
}
