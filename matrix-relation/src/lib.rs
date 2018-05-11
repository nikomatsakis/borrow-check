#![feature(catch_expr)]
#![feature(const_fn)]
#![feature(crate_in_paths)]
#![feature(crate_visibility_modifier)]
#![feature(proc_macro)]
#![feature(extern_prelude)]
#![feature(extern_absolute_paths)]
#![feature(in_band_lifetimes)]
#![feature(termination_trait_test)]
#![feature(macro_vis_matcher)]
#![feature(nonzero)]

extern crate fxhash;

mod bitvec;
mod indexed_vec;
mod test;

use crate::bitvec::{SparseBitMatrix, SparseBitSet, SparseChunk};
use crate::indexed_vec::Idx;
use fxhash::FxHashMap;
use std::hash::Hash;

/// A graph data struture that preserve transitive reachability relationships.
///
/// For example, if we have a graph:
///
/// ```notrust
/// A --> B
/// B --> C
/// ```
///
/// Upon removing `B`, we preserve that there exists a path from `A` to `C`, and so
/// our graph becomes:
/// ```notrust
/// A --> C
/// ```
#[derive(Debug)]
pub struct Relation<R: Idx + Hash> {
    adjacency: SparseBitMatrix<R, R>,
}

impl<R: Idx + Hash> Relation<R> {
    pub fn new(rows: usize) -> Relation<R> {
        Relation {
            adjacency: SparseBitMatrix::new(R::new(rows), R::new(rows)),
        }
    }

    pub fn add_edge(&mut self, row1: R, row2: R) -> bool {
        self.adjacency.add(row1, row2)
    }

    #[cfg(test)]
    fn kill(&mut self, live_nodes: &[R], dead_nodes: &[R]) {
        let mut dead_bits = SparseBitSet::new();
        for &n in dead_nodes {
            assert!(!live_nodes.contains(&n));
            dead_bits.insert_chunk(SparseChunk::one(n));
        }
        self.remove_dead_nodes(live_nodes, &dead_bits)
    }

    pub fn remove_dead_nodes(&mut self, live_nodes: &[R], dead_nodes: &SparseBitSet<R>) {
        // First operation:
        //
        // - For each live region R1 that can reach dead-nodes:
        //   - Find R2 = Adj(R1) & D
        //   -
        //
        // Once all this is done, we remove dead nodes.

        let mut live_targets: FxHashMap<R, SparseBitSet<R>> = FxHashMap::default();

        for &live_source in live_nodes {
            for dead_chunk in dead_nodes.chunks() {
                let dead_targets = self.adjacency.row(live_source).contains_chunk(dead_chunk);
                if !dead_targets.any() {
                    continue;
                }

                for dead_target in dead_targets.iter() {
                    // For each dead target, we have to find all the
                    // live nodes reachable from it. Those will get
                    // added to the row for `live_source`.
                    let live_target_set = live_targets.entry(dead_target).or_insert_with(|| {
                        self.find_live_targets(dead_target, dead_nodes)
                    });

                    self.adjacency
                        .row_mut(live_source)
                        .insert_chunks(live_target_set);
                }

                // Clear out the dead things.
                self.adjacency.row_mut(live_source)
                    .remove_chunk(dead_chunk);
            }
        }

        for dead_node in dead_nodes.iter() {
            *self.adjacency.row_mut(dead_node) = SparseBitSet::new();
        }
    }

    fn find_live_targets(
        &self,
        dead_target: R,
        dead_nodes: &SparseBitSet<R>,
    ) -> SparseBitSet<R> {
        let mut result = SparseBitSet::new();
        result.insert_chunk(SparseChunk::one(dead_target));
        let mut queue = vec![SparseChunk::one(dead_target)];
        while let Some(dead_targets) = queue.pop() {
            // Invariant: All the nodes in `dead_targets` are dead.
            assert!(
                dead_nodes
                    .contains_chunk(dead_targets)
                    .bits_eq(dead_targets)
            );

            // For each dead region `Rd`
            for dead_target in dead_targets.iter() {
                // Find those things `N` reachable from `Rd`.
                let next_nodes = self.adjacency.row(dead_target);

                for next_chunk in next_nodes.chunks() {
                    // Add all the things reachable things to the result.
                    // Track the things that were not present.
                    let new_chunk = result.insert_chunk(next_chunk);

                    // Find those new things that are dead and enqueue.
                    let new_dead_targets = dead_nodes.contains_chunk(new_chunk);
                    queue.push(new_dead_targets);
                }
            }
        }

        // Finally, remove the dead things.
        for dead_chunk in dead_nodes.chunks() {
            result.remove_chunk(dead_chunk);
        }

        result
    }

    #[cfg(test)]
    fn dump_and_assert(&self) -> Vec<String> {
        let mut result = vec![];

        for (index, row) in self.adjacency.rows().enumerate() {
            let pred = R::new(index);
            for succ in row.iter() {
                result.push(format!("{:?} --> {:?}", pred, succ));
            }
        }

        result
    }
}
