#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Key {
    pub(crate) id: usize,
}

impl Key {
    #[allow(dead_code)]
    pub(crate) fn new(id: usize) -> Self {
        Self { id }
    }

    #[must_use]
    /// Return the ID of the entity this key correspond
    pub fn id(self) -> usize {
        self.id
    }

    #[allow(dead_code)]
    pub fn dummy() -> Self {
        Self { id: usize::MAX }
    }
}

// #[derive(Debug)]
// pub struct StateKey<T> {
//     pub(crate) id: usize,
//     pub(crate) value: PhantomData<T>,
// }

// impl<T> Clone for StateKey<T> {
//     fn clone(&self) -> Self {
//         Self {
//             id: self.id,
//             value: PhantomData,
//         }
//     }
// }

// impl<T> Copy for StateKey<T> {}

// impl<V> StateKey<V> {
//     #[must_use]
//     pub(crate) fn new(id: usize) -> Self {
//         let value = PhantomData;
//         Self { id, value }
//     }

//     #[must_use]
//     pub fn id(self) -> usize {
//         self.id
//     }

//     #[allow(dead_code)]
//     pub(crate) fn new_unchecked(id: usize) -> Self {
//         let value = PhantomData;
//         Self { id, value }
//     }
// }
