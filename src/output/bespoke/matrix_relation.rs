use crate::facts::Region;
use crate::output::bespoke::SubsetRelation;
use fxhash::FxHashSet;
use matrix_relation::bitvec::SparseBitSet;
use matrix_relation::{indexed_vec::Idx, Relation};
use std::collections::BTreeSet;

pub struct MatrixRelation {
    data: Relation<Region>,
}

impl Idx for Region {
    fn new(idx: usize) -> Self {
        Region::from(idx)
    }

    fn index(self) -> usize {
        self.into()
    }
}

impl Clone for MatrixRelation {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl SubsetRelation for MatrixRelation {
    fn empty(num_regions: usize) -> Self {
        Self {
            data: Relation::new(num_regions),
        }
    }

    fn kill_region(
        &mut self,
        live_regions: impl Iterator<Item = Region>,
        dead_regions: &SparseBitSet<Region>,
    ) {
        self.data.remove_dead_nodes(live_regions, dead_regions)
    }

    fn insert_one(&mut self, r1: Region, r2: Region) -> bool {
        self.data.add_edge(r1, r2)
    }

    fn insert_all(&mut self, other: &Self, live_regions: &BTreeSet<Region>) -> bool {
        self.data.add_rows(&other.data, live_regions.iter().cloned())
    }

    fn for_each_reachable(&self, r1: Region, mut op: impl FnMut(Region)) {
        let mut stack = vec![r1];
        let mut visited = FxHashSet::default();
        visited.insert(r1);

        while let Some(p) = stack.pop() {
            op(p);
            for s in self.data.successors(p) {
                if visited.insert(s) {
                    stack.push(s);
                }
            }
        }
    }
}
