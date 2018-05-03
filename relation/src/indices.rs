use crate::index_vec::IndexType;
use crate::Direction;
use std::num::NonZeroU32;
use std::ops::{Index, IndexMut};
use std::fmt;

macro_rules! index_type {
    ($v:vis struct $n:ident { prefix: $prefix:expr }) => {
        #[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
        $v struct $n {
            value: NonZeroU32
        }

        impl From<usize> for $n {
            fn from(value: usize) -> $n {
                assert!(value < (u32::max_value() as usize));
                $n { value: NonZeroU32::new((value as u32) + 1).unwrap() }
            }
        }

        impl IndexType for $n {
            fn to_usize(self) -> usize {
                (self.value.get() as usize) - 1
            }
        }

        impl fmt::Debug for $n {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "{}({})", $prefix, self.to_usize())
            }
        }
    }
}

index_type! {
    pub struct EdgeIndex { prefix: "E" }
}

index_type! {
    pub struct NodeIndex { prefix: "N" }
}

#[derive(Copy, Clone, Default, Debug)]
crate struct Indices<N> {
    values: (N, N),
}

impl<N> Indices<N> {
    crate fn new(incoming: N, outgoing: N) -> Self {
        Indices {
            values: (incoming, outgoing),
        }
    }

    crate fn incoming(&self) -> N
    where
        N: Copy,
    {
        self.values.0
    }

    crate fn set_incoming(&mut self, value: N) {
        self.values.0 = value;
    }

    crate fn outgoing(&self) -> N
    where
        N: Copy,
    {
        self.values.1
    }

    crate fn set_outgoing(&mut self, value: N) {
        self.values.1 = value;
    }
}

impl<N> Indices<Option<N>> {
    crate fn take_outgoing(&mut self) -> Option<N> {
        self.values.1.take()
    }
}

impl<N> Index<Direction> for Indices<N> {
    type Output = N;

    fn index(&self, direction: Direction) -> &Self::Output {
        match direction {
            Direction::Incoming => &self.values.0,
            Direction::Outgoing => &self.values.1,
        }
    }
}

impl<N> IndexMut<Direction> for Indices<N> {
    fn index_mut(&mut self, direction: Direction) -> &mut Self::Output {
        match direction {
            Direction::Incoming => &mut self.values.0,
            Direction::Outgoing => &mut self.values.1,
        }
    }
}
