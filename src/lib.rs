#![feature(generators, generator_trait)]
// use std::cell::Cell;

mod container;
mod keys;
mod scheduler;
mod simulation;
mod state;

use std::{ops::Generator, time::Duration};

pub use keys::Key;
pub use simulation::{Simulation, ShouldContinue};
pub use state::{State, StateKey};

pub type GenBoxed<R, C = ()> = Box<dyn Generator<R, Yield = Action, Return = C> + Unpin>;

// Action Define que acción realiza la simulación
// Este enum es devuelto tras ejecutar un step de los generadores
#[derive(Debug, Clone)]
pub enum Action {
    Hold(Duration),
    Passivate,
    ActivateOne(Key),
    ActivateMany(Vec<Key>),
    Cancel(Key),
}

impl Action {
    #[inline]
    pub fn activate_one(key: Key) -> Self {
        Action::ActivateOne(key)
    }
    #[inline]
    pub fn activate_many(keys: Vec<Key>) -> Self {
        Action::ActivateMany(keys)
    }
}

// thread_local! {
//     static ID_COUNTER: Cell<usize> = Cell::new(0);
// }

// // #[tracing::instrument]
// fn generate_next_id() -> usize {
//     // use tracing::trace;
//     // let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
//     let id = ID_COUNTER.with(|cell| {
//         let id = cell.get();
//         cell.set(id + 1);
//         id
//     });
//     // trace!("Generating new Id = {}", id);
//     id
// }
