use std::cell::Cell;
use std::ops::GeneratorState;
use std::rc::Rc;
use std::time::Duration;

use crate::container::{Container, EntityState};
use crate::scheduler::Scheduler;
use crate::state::State;
use crate::{Action, GenBoxed, Key};

pub struct Simulation<R> {
    scheduler: Scheduler,
    entities: Container<R>,
    state: Rc<Cell<State>>,
}

pub enum ShouldContinue {
    Advance,
    Break,
}

impl<R> Default for Simulation<R>
where
    R: 'static,
{
    fn default() -> Self {
        Self {
            scheduler: Scheduler::default(),
            entities: Container::default(),
            state: Rc::new(Cell::new(State::default()))
        }
    }
}

impl<R> Simulation<R>
where
    R: 'static,
{
    /// Add an already constructed Generator into the simulation.
    #[inline]
    pub fn add_generator(&mut self, gen: GenBoxed<R>) -> Key {
        self.entities.add_generator(gen)
    }

    /// Schedules `entity_key` at `self.time() + time`.
    /// 
    /// `entity_key` is a [Key] corresponding to the entity to be scheduled.
    /// 
    /// If `entity_key` was already scheduled it will ignore the following calls
    #[inline]
    pub fn schedule(&mut self, time: Duration, entity_key: Key) {
        self.scheduler.schedule(time, entity_key)
    }

    /// Schedules `entity_key` to be executed for at `self.time()`.
    ///
    /// the `entity_key` argument is a [`Key`] corresponding to the [Generator](crate::GenBoxed) to be scheduled.
    /// 
    /// If `entity_key` was already scheduled it will ignore the following calls
    #[inline]
    pub fn schedule_now(&mut self, entity_key: Key) {
        self.scheduler.schedule_now(entity_key)
    }

    /// Returns the current simulation time.
    #[must_use]
    #[inline]
    pub fn time(&self) -> Duration {
        self.scheduler.time()
    }

    #[must_use]
    #[inline]
    pub fn clock(&self) -> crate::scheduler::ClockRef {
        self.scheduler.clock()
    }

    /// Retrieve a copy of the current [EntityState] of the generator asociated with `key`
    #[must_use]
    pub fn entity_state(&self, key: Key) -> Option<EntityState> {
        self.entities.get_state(key).copied()
    }

    /// Advance the simulation one event.
    pub fn step_with(&mut self, resume_with: R) -> ShouldContinue {
        if let Some(event_entry) = self.scheduler.pop() {
            let key = event_entry.key();

            let state = self.entities.step_with(key, resume_with);
            match state {
                GeneratorState::Yielded(action) => {
                    let entity_state = self.entities.get_state_mut(key).unwrap();
                    match action {
                        Action::Hold(duration) => {
                            // TODO: Maybe remove this check. It shouldn't happen.
                            if let EntityState::Passive = *entity_state {
                                panic!(
                                    "A passive entity received a hold command. ID = {}",
                                    key.id
                                );
                            }
                            self.schedule(duration, key);
                        }
                        Action::Passivate => {
                            // TODO: This check shouldn't happen, a passive generator
                            // shouldn't be able to send another passivate
                            match *entity_state {
                                EntityState::Active => {
                                    *entity_state = EntityState::Passive;
                                }
                                EntityState::Passive => {
                                    panic!(
                                        "A passive entity received a passivate command. ID = {}",
                                        key.id
                                    );
                                }
                            }
                        }
                        Action::ActivateOne(other_key) => {
                            // TODO: This check shouldn't be necessary a passive generator
                            // shouldn't be able to send an activate.
                            if let EntityState::Passive = *entity_state {
                                panic!("A passive entity sended an activate. ID = {}", key.id);
                            }
                            self.schedule_now(key);

                            let other_state = self.entities.get_state_mut(other_key).unwrap();
                            match *other_state {
                                EntityState::Passive => {
                                    *other_state = EntityState::Active;
                                }
                                EntityState::Active => {
                                    panic!(
                                        "Entity ID = {} tried to Activate Entity ID = {} but it was already active",
                                        key.id,
                                        other_key.id
                                    )
                                }
                            }

                            self.schedule_now(other_key);
                        }
                        Action::ActivateMany(other_keys) => {
                            if let EntityState::Passive = *entity_state {
                                panic!("A passive entity sended an activate. ID = {}", key.id);
                            }
                            self.schedule_now(key);
                            for other_key in other_keys {
                                let other_state = self.entities.get_state_mut(other_key).unwrap();
                                match *other_state {
                                    EntityState::Passive => {
                                        *other_state = EntityState::Active;
                                    }
                                    EntityState::Active => {
                                        panic!(
                                            "Entity ID = {} tried to Activate Entity ID = {} but it was already active",
                                            key.id,
                                            other_key.id
                                        )
                                    }
                                }
                                self.schedule_now(other_key);
                            }
                        }
                        Action::Cancel(other_key) => {
                            if let EntityState::Passive = *entity_state {
                                panic!(
                                    "A passive entity did a Cancel. ID = {} to ID = {}",
                                    key.id, other_key.id
                                );
                            }
                            self.schedule_now(key);
                            
                            // -----------------------------------
                            let other_state = self.entities.get_state_mut(other_key).unwrap();
                            match *other_state {
                                EntityState::Active => {
                                    *other_state = EntityState::Passive;
                                }
                                EntityState::Passive => {
                                    panic!(
                                        "Entity ID = {} sent Cancel to Entity ID = {} but is was in a passive state",
                                        key.id,
                                        other_key.id
                                    )
                                }
                            }
                            // TODO: PROFILE AND OPTIMIZE THIS ENTIRE CHUNK

                            // TODO: Maybe remove this check because if it passed the previous check then an event is guaranteed to exist in the scheduler
                            // ---------------
                            if !self.scheduler.remove(other_key) {
                                panic!("Entity ID = {} send Cancel to ID = {} and it wasn't scheduled", key.id, other_key.id);
                            };
                            // ---------------
                        }
                    }
                }
                GeneratorState::Complete(_) => {
                    self.entities.remove(key);
                }
            }
            ShouldContinue::Advance
        } else {
            ShouldContinue::Break
        }
    }

    pub fn state(&self) -> Rc<Cell<State>> {
        Rc::clone(&self.state)
    }
}

impl Simulation<()> {
    #[inline]
    pub fn step(&mut self) -> ShouldContinue {
        self.step_with(())
    }

    pub fn run_until_empty(&mut self) {
        while let ShouldContinue::Advance = self.step() {}
    }

    pub fn run_with_limit(&mut self, limit: Duration) {
        while let ShouldContinue::Advance = self.step() {
            if self.time() >= limit {
                break;
            }
        }
    }
}
