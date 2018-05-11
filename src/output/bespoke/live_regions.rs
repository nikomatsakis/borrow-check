use crate::facts::{AllFacts, Point, Region};
use crate::intern::InternerTables;
use fxhash::FxHashMap;
use matrix_relation::bitvec::{SparseBitSet, SparseChunk};
use std::collections::BTreeSet;

crate struct LiveRegions {
    live_regions: Vec<BTreeSet<Region>>,
    active_regions: Vec<BTreeSet<Region>>,
    dying_regions: FxHashMap<(Point, Point), SparseBitSet<Region>>,
}

impl LiveRegions {
    crate fn from(tables: &InternerTables, all_facts: &AllFacts) -> Self {
        let num_points = tables.len::<Point>();

        // Compute what is live (or may contain points) at each point.
        let mut live_regions: Vec<_> = (0..num_points).map(|_| BTreeSet::new()).collect();
        for (region, point) in &all_facts.region_live_at {
            live_regions[point.index()].insert(*region);
        }

        let mut active_regions = live_regions.clone();
        for (r1, r2, point) in &all_facts.outlives {
            let mut set = &mut active_regions[point.index()];
            set.insert(*r1);
            set.insert(*r2);
        }

        let mut dying_regions = FxHashMap::default();
        for &(p, q) in &all_facts.cfg_edge {
            let mut bit_set = SparseBitSet::new();
            let active_at_p = &active_regions[p.index()];
            let live_at_q = &live_regions[q.index()];
            for r in active_at_p
                .iter()
                .cloned()
                .filter(move |r| !live_at_q.contains(r))
            {
                bit_set.insert_chunk(SparseChunk::one(r));
            }
            dying_regions.insert((p, q), bit_set);
        }

        LiveRegions {
            live_regions,
            active_regions,
            dying_regions,
        }
    }

    crate fn live_at(&self, point: Point, region: Region) -> bool {
        self.live_regions[point.index()].contains(&region)
    }

    crate fn live_regions_at(&self, point: Point) -> &BTreeSet<Region> {
        &self.live_regions[point.index()]
    }

    crate fn dying_on_edge(&self, p: Point, q: Point) -> Option<&SparseBitSet<Region>> {
        self.dying_regions.get(&(p, q))
    }
}
