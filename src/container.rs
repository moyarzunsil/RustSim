use crate::{keys::Key, Action, GenBoxed};
use std::ops::GeneratorState;
use std::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityState {
    Passive,
    Active,
}

pub struct Container<R> {
    pub(crate) inner: Vec<Option<(GenBoxed<R>, EntityState)>>,
}

impl<R> Default for Container<R>
where
    R: 'static,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<R> Container<R>
where
    R: 'static,
{
    pub fn add_generator(&mut self, gen: GenBoxed<R>) -> Key {
        let key = Key::new(self.inner.len());
        self.inner.push(Some((gen, EntityState::Active)));
        key
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, key: Key) -> Option<(GenBoxed<R>, EntityState)> {
        // if self.inner.get(key.id).is_some() {
        //     self.inner[key.id].take()
        // } else {
        //     None
        // }

        // Another way of doing the above added in rust 1.62
        // self.inner.get(key.id).is_some().then_some(self.inner[key.id].take()).flatten()

        self.inner.get_mut(key.id).and_then(Option::take)
    }

    /// Returns the number of elements in the container.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the container contains no elements.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Advance the entity defined by `key`
    ///
    /// # Panics
    ///
    /// Panics when the key used was for an already extracted generator
    /// or if the generator has already completed its execution.
    pub fn step_with(&mut self, key: Key, resume_with: R) -> GeneratorState<Action, ()> {
        // Esto asume que los eventos nunca son borrados.
        // TODO: Confirmar esta asumpción.

        let &mut (ref mut gen, _) = self
            .inner
            .get_mut(key.id)
            .and_then(Option::as_mut)
            .expect("entities shouldn't be removed from the container");

        // gen.step(resume_with)
        let gen = gen.as_mut();
        Pin::new(gen).resume(resume_with)
        // gen.resume_with(resume_with)
    }

    #[must_use]
    pub fn get_state(&self, key: Key) -> Option<&EntityState> {
        // if let Some(values) = self.inner.get(key.id) {
        //     values.as_ref().map(|(_, ref state)| state)
        // } else {
        //     None
        // }

        self.inner
            .get(key.id)
            .and_then(Option::as_ref)
            .map(|(_, state)| state)
    }

    #[must_use]
    pub fn get_state_mut(&mut self, key: Key) -> Option<&mut EntityState> {
        // if let Some(value) = self.inner.get_mut(key.id) {
        //     value.as_mut().map(|&mut (_, ref mut state)| state)
        // } else {
        //     None
        // }

        self.inner
            .get_mut(key.id)
            .and_then(Option::as_mut)
            .map(|&mut (_, ref mut state)| state)
    }
}

impl Container<()> {
    #[allow(dead_code)]
    pub fn step(&mut self, key: Key) -> GeneratorState<Action, ()> {
        self.step_with(key, ())
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;

    fn producer(kind: &'static str) -> GenBoxed<()> {
        let gen = move |_| {
            println!("Iniciando {}", kind);
            // TODO: FIX THIS FUNCION. ESPECIFICAMENTE EL TIPO DE YIELD
            yield Action::Passivate;
            for i in 0..3 {
                println!(
                    "{} ha sido llamado {} {}",
                    kind,
                    i + 1,
                    if i == 0 { "vez" } else { "veces" }
                );
                yield Action::Passivate;
            }
            println!("{} Finaliza", kind);
        };
        Box::new(gen)
    }

    fn finite(name: &'static str, number_of_loops: u8) -> GenBoxed<()> {
        let gen = move |_| {
            for i in 0..number_of_loops {
                println!("Yield");
                let _ = yield Action::Hold(Duration::ZERO);
                // co.hold(Duration::ZERO).await
                println!("{} has yielded {} times", name, i + 1);
            }
            println!("{} completed", name);
        };
        Box::new(gen)
    }

    fn infinite(indentifier: usize) -> GenBoxed<()> {
        let gen = move |_| {
            println!("This function is starting and will never complete");
            let mut i = 1;
            loop {
                println!(
                    "Infinite Generator N°{} is Yielding | It has Yielded {} times",
                    indentifier, i
                );
                let _ = yield Action::Hold(Duration::ZERO);
                // co.hold(Duration::ZERO).await;
                i += 1;
            }
        };
        Box::new(gen)
    }

    #[test]
    fn generators_can_be_inserted() {
        let mut container = Container::default();
        // Assert that the container is empty
        assert!(container.is_empty());
        // Creating and inserting a generator to the container
        let gen = producer("A");
        let first_key = container.add_generator(gen);
        assert_eq!(0, first_key.id());
        // Same as above but inline
        let second_key = container.add_generator(producer("B"));
        assert_eq!(1, second_key.id());
        // A different function can be converted to a generator and inserted to the container
        let gen = finite("A", 42);
        let third_key = container.add_generator(gen);
        assert_eq!(2, third_key.id());
        // as long as the types of the returned GenBoxed match
        let fourth_key = container.add_generator(infinite(1));
        assert_eq!(3, fourth_key.id());
        // Assert that all generators were inserted correctly to the container.
        assert_eq!(4, container.len());
    }

    #[test]
    fn generators_can_be_resumed() {
        let mut container = Container::default();
        // Using the finite function because if infinite was used in its place this test would never end.
        let finite_key = container.add_generator(finite("A", 3));
        
        while let GeneratorState::Yielded(_) = container.step_with(finite_key, ()) {}

        // Uncommenting the following line will cause the test to fail.
        // container.step_with(finite_key, ());
        // This is because when a generator completes, to say, the original function end its excecution
        // The generator cannot be resumed again and it's an error to do so.
    }   
}
