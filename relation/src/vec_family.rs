use crate::indices::{EdgeIndex, NodeIndex};
use crate::{EdgeData, NodeData};
use std; // FIXME
use std::fmt::Debug;
use std::hash::Hash;

pub trait VecFamily: Debug + Default + Sized {
    type UserNode: Debug;
    type Node: IndexType;
    type Edge: IndexType;
    type NodeVec: IndexVec<Self::Node, NodeData<Self>>;
    type EdgeVec: IndexVec<Self::Edge, EdgeData<Self>>;

    fn into_node(Self::UserNode) -> Self::Node;
}

pub trait IndexType: Copy + Debug + Ord + Eq + Hash + From<usize> {
    fn to_usize(self) -> usize;
}

pub trait IndexVec<I, T>: Default + Debug
where
    I: IndexType,
    T: Debug,
{
    fn with_default_elements(num_elts: usize) -> Self
    where
        T: Default;
    fn empty() -> Self;
    fn get(&self, index: I) -> &T;
    fn get_mut(&mut self, index: I) -> &mut T;
    fn set(&mut self, index: I, value: T);
    fn push(&mut self, value: T) -> I;
    fn len(&self) -> usize;
}

impl<I, T> IndexVec<I, T> for Vec<T>
where
    I: IndexType,
    T: Debug,
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

    fn len(&self) -> usize {
        self.len()
    }
}

pub struct StdVec<U> {
    data: std::marker::PhantomData<U>
}

impl<U> Default for StdVec<U> {
    fn default() -> Self {
        Self { data: std::marker::PhantomData }
    }
}

impl<U> Debug for StdVec<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "StdVec")
    }
}

impl<U: Into<usize> + Debug> VecFamily for StdVec<U> {
    type UserNode = U;
    type Node = NodeIndex;
    type Edge = EdgeIndex;
    type NodeVec = Vec<NodeData<Self>>;
    type EdgeVec = Vec<EdgeData<Self>>;

    fn into_node(u: U) -> NodeIndex {
        let u: usize = u.into();
        NodeIndex::from(u)
    }
}

