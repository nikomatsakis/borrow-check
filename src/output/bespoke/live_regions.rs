use crate::facts::{AllFacts, Point, Region};
use crate::intern::InternerTables;
use std::collections::BTreeSet;

crate struct LiveRegions {
    live_regions: Vec<BTreeSet<Region>>,
    active_regions: Vec<BTreeSet<Region>>,
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

        LiveRegions {
            live_regions,
            active_regions,
        }
    }

    crate fn live_at(&self, point: Point, region: Region) -> bool {
        self.live_regions[point.index()].contains(&region)
    }

    crate fn live_regions_at(&self, point: Point) -> &BTreeSet<Region> {
        &self.live_regions[point.index()]
    }

    crate fn dying_on_edge(&self, p: Point, q: Point) -> impl Iterator<Item = Region> + '_ {
        let active_at_p = &self.active_regions[p.index()];
        let live_at_q = &self.live_regions[q.index()];
        active_at_p.iter().cloned().filter(move |r| !live_at_q.contains(r))
    }
}
