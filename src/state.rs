use std::marker::PhantomData;

#[derive(Debug)]
pub struct StateKey<T> {
    id: usize,
    value: PhantomData<T>,
}

impl<T> Clone for StateKey<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: PhantomData,
        }
    }
}

impl<T> Copy for StateKey<T> {}

impl<V> StateKey<V> {
    #[must_use]
    fn new(id: usize) -> Self {
        let value = PhantomData;
        Self { id, value }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn id(self) -> usize {
        self.id
    }
}

use std::any::Any;

#[derive(Debug, Default)]
pub struct State {
    store: Vec<Option<Box<dyn Any>>>,
}

impl State {
    pub fn insert<V: 'static>(&mut self, value: V) -> StateKey<V> {
        let id = self.store.len();
        self.store.push(Some(Box::new(value)));
        StateKey::new(id)
    }

    #[allow(dead_code)]
    pub fn remove<V: 'static>(&mut self, key: StateKey<V>) -> Option<V> {
        // if self.store.get(key.id).is_some() {
        //     self.store[key.id]
        //         .take()
        //         .map(|value| *value.downcast::<V>().expect("Ensured by the Key type."))
        // } else {
        //     None
        // }

        // if let Some(key) = self.store.get_mut(key.id) {
        //     key.take()
        //         .map(|value| *value.downcast::<V>().expect("Ensured by the Key type."))
        // } else {
        //     None
        // }

        self.store
            .get_mut(key.id)
            .and_then(Option::take)
            .map(|value| *value.downcast::<V>().expect("Ensured by the Key type."))
    }

    pub fn get<V: 'static>(&self, key: StateKey<V>) -> Option<&V> {
        // if let Some(value) = self.store.get(key.id) {
        //     value.map(|value| value.downcast_ref::<V>().expect("Ensured by the key type."))
        // } else {
        //     None
        // }

        // The code above and bellow are identical in meaning
        // Performance of both is not tested in any way
        // Which of both is clearer remains to be seen.

        self.store
            .get(key.id)
            .and_then(Option::as_ref)
            .map(|value| value.downcast_ref::<V>().expect("Ensured by the key type."))
    }

    pub fn get_mut<V: 'static>(&mut self, key: StateKey<V>) -> Option<&mut V> {
        // if let Some(value) = self.store.get_mut(key.id) {
        //     value.map(|value| value.downcast_mut::<V>().expect("Ensured by the key type."))
        // } else {
        //     None
        // }

        // The code above and bellow are identical in meaning
        // Performance of both is not tested in any way
        // Which of both is clearer remains to be seen.

        self.store
            .get_mut(key.id)
            .and_then(Option::as_mut)
            .map(|value| value.downcast_mut::<V>().expect("Ensured by the key type."))
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}
