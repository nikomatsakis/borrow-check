use crate::indices::{EdgeIndex, NodeIndex};
use crate::{EdgeData, NodeData};
use std::hash::Hash;

pub trait VecFamily {
    type NodeVec: IndexVec<NodeIndex, NodeData>;
    type EdgeVec: IndexVec<EdgeIndex, EdgeData>;
}

pub trait IndexType: Copy + Ord + Eq + Hash + From<usize> {
    fn to_usize(self) -> usize;
}

pub trait IndexVec<I, T>
where
    I: IndexType,
{
    fn with_default_elements(num_elts: usize) -> Self
    where
        T: Default;
    fn empty() -> Self;
    fn get(&self, index: I) -> &T;
    fn get_mut(&mut self, index: I) -> &mut T;
    fn set(&mut self, index: I, value: T);
    fn push(&mut self, value: T) -> I;
}

impl<I, T> IndexVec<I, T> for Vec<T>
where
    I: IndexType,
{
    fn with_default_elements(num_elts: usize) -> Self
    where
        T: Default,
    {
        (0..num_elts).map(|_| T::default()).collect()
    }

    fn empty() -> Self {
        Self::new()
    }

    fn get(&self, index: I) -> &T {
        &self[index.to_usize()]
    }

    fn get_mut(&mut self, index: I) -> &mut T {
        &mut self[index.to_usize()]
    }

    fn set(&mut self, index: I, value: T) {
        self[index.to_usize()] = value;
    }

    fn push(&mut self, value: T) -> I {
        let len = self.len();
        self.push(value);
        I::from(len)
    }
}
