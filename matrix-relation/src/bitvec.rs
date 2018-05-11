// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

use indexed_vec::{Idx, IndexVec};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::marker::PhantomData;

type Word = u128;

#[derive(Clone, Debug)]
pub struct SparseBitMatrix<R, C>
where
    R: Idx,
    C: Idx,
{
    vector: IndexVec<R, SparseBitSet<C>>,
}

impl<R: Idx, C: Idx> SparseBitMatrix<R, C> {
    /// Create a new `rows x columns` matrix, initially empty.
    pub fn new(rows: R, _columns: C) -> SparseBitMatrix<R, C> {
        SparseBitMatrix {
            vector: IndexVec::from_elem_n(SparseBitSet::new(), rows.index()),
        }
    }

    /// Sets the cell at `(row, column)` to true. Put another way, insert
    /// `column` to the bitset for `row`.
    ///
    /// Returns true if this changed the matrix, and false otherwise.
    pub fn add(&mut self, row: R, column: C) -> bool {
        self.vector[row].insert(column)
    }

    /// Do the bits from `row` contain `column`? Put another way, is
    /// the matrix cell at `(row, column)` true?  Put yet another way,
    /// if the matrix represents (transitive) reachability, can
    /// `row` reach `column`?
    pub fn contains(&self, row: R, column: C) -> bool {
        self.vector[row].contains(column)
    }

    /// Add the bits from row `read` to the bits from row `write`,
    /// return true if anything changed.
    ///
    /// This is used when computing transitive reachability because if
    /// you have an edge `write -> read`, because in that case
    /// `write` can reach everything that `read` can (and
    /// potentially more).
    pub fn merge(&mut self, read: R, write: R) -> bool {
        let mut changed = false;

        if read != write {
            let (bit_set_read, bit_set_write) = self.vector.pick2_mut(read, write);

            for read_chunk in bit_set_read.chunks() {
                changed = changed | bit_set_write.insert_chunk(read_chunk).any();
            }
        }

        changed
    }

    /// True if `sub` is a subset of `sup`
    pub fn is_subset(&self, sub: R, sup: R) -> bool {
        sub == sup || {
            let bit_set_sub = &self.vector[sub];
            let bit_set_sup = &self.vector[sup];
            bit_set_sub
                .chunks()
                .all(|read_chunk| read_chunk.bits_eq(bit_set_sup.contains_chunk(read_chunk)))
        }
    }

    /// Iterates through all the columns set to true in a given row of
    /// the matrix.
    pub fn iter<'a>(&'a self, row: R) -> impl Iterator<Item = C> + 'a {
        self.vector[row].iter()
    }

    pub fn rows<'a>(&'a self) -> impl Iterator<Item = &'a SparseBitSet<C>> + 'a {
        self.vector.iter()
    }

    pub fn row(&self, row: R) -> &SparseBitSet<C> {
        &self.vector[row]
    }

    pub fn row_mut(&mut self, row: R) -> &mut SparseBitSet<C> {
        &mut self.vector[row]
    }
}

#[derive(Clone, Debug)]
pub struct SparseBitSet<I: Idx> {
    chunk_bits: BTreeMap<u32, Word>,
    _marker: PhantomData<I>,
}

#[derive(Copy, Clone)]
pub struct SparseChunk<I> {
    key: u32,
    bits: Word,
    _marker: PhantomData<I>,
}

impl<I: Idx> SparseChunk<I> {
    #[inline]
    pub fn one(index: I) -> Self {
        let index = index.index();
        let key_usize = index / 128;
        let key = key_usize as u32;
        assert_eq!(key as usize, key_usize);
        SparseChunk {
            key,
            bits: 1 << (index % 128),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.bits != 0
    }

    #[inline]
    pub fn bits_eq(&self, other: SparseChunk<I>) -> bool {
        self.bits == other.bits
    }

    pub fn iter(&self) -> impl Iterator<Item = I> {
        let base = self.key as usize * 128;
        let mut bits = self.bits;
        (0..128)
            .map(move |i| {
                let current_bits = bits;
                bits >>= 1;
                (i, current_bits)
            })
            .take_while(|&(_, bits)| bits != 0)
            .filter_map(move |(i, bits)| {
                if (bits & 1) != 0 {
                    Some(I::new(base + i))
                } else {
                    None
                }
            })
    }
}

impl<I: Idx> SparseBitSet<I> {
    pub fn new() -> Self {
        SparseBitSet {
            chunk_bits: BTreeMap::new(),
            _marker: PhantomData,
        }
    }

    pub fn capacity(&self) -> usize {
        self.chunk_bits.len() * 128
    }

    /// Returns a chunk containing only those bits that are already
    /// present. You can test therefore if `self` contains all the
    /// bits in chunk already by doing `chunk ==
    /// self.contains_chunk(chunk)`.
    pub fn contains_chunk(&self, chunk: SparseChunk<I>) -> SparseChunk<I> {
        SparseChunk {
            bits: self.chunk_bits
                .get(&chunk.key)
                .map_or(0, |bits| bits & chunk.bits),
            ..chunk
        }
    }

    /// Modifies `self` to contain all the bits from `chunk` (in
    /// addition to any pre-existing bits); returns a new chunk that
    /// contains only those bits that were newly added. You can test
    /// if anything was inserted by invoking `any()` on the returned
    /// value.
    pub fn insert_chunk(&mut self, chunk: SparseChunk<I>) -> SparseChunk<I> {
        if chunk.bits == 0 {
            return chunk;
        }
        let bits = self.chunk_bits.entry(chunk.key).or_insert(0);
        let old_bits = *bits;
        let new_bits = old_bits | chunk.bits;
        *bits = new_bits;
        let changed = new_bits ^ old_bits;
        SparseChunk {
            bits: changed,
            ..chunk
        }
    }

    pub fn insert_chunks(&mut self, other: &SparseBitSet<I>) -> bool {
        let mut changed = false;
        for chunk in other.chunks() {
            changed |= self.insert_chunk(chunk).any();
        }
        changed
    }

    pub fn remove_chunk(&mut self, chunk: SparseChunk<I>) -> SparseChunk<I> {
        if chunk.bits == 0 {
            return chunk;
        }
        let changed = match self.chunk_bits.entry(chunk.key) {
            Entry::Occupied(mut bits) => {
                let old_bits = *bits.get();
                let new_bits = old_bits & !chunk.bits;
                if new_bits == 0 {
                    bits.remove();
                } else {
                    bits.insert(new_bits);
                }
                new_bits ^ old_bits
            }
            Entry::Vacant(_) => 0,
        };
        SparseChunk {
            bits: changed,
            ..chunk
        }
    }

    pub fn clear(&mut self) {
        self.chunk_bits.clear();
    }

    pub fn chunks<'a>(&'a self) -> impl Iterator<Item = SparseChunk<I>> + 'a {
        self.chunk_bits.iter().map(|(&key, &bits)| SparseChunk {
            key,
            bits,
            _marker: PhantomData,
        })
    }

    pub fn contains(&self, index: I) -> bool {
        self.contains_chunk(SparseChunk::one(index)).any()
    }

    pub fn insert(&mut self, index: I) -> bool {
        self.insert_chunk(SparseChunk::one(index)).any()
    }

    pub fn remove(&mut self, index: I) -> bool {
        self.remove_chunk(SparseChunk::one(index)).any()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = I> + 'a {
        self.chunks().flat_map(|chunk| chunk.iter())
    }
}
