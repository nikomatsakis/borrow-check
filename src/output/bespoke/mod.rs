// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::facts::{AllFacts, Point, Region};
use crate::intern::InternerTables;
use crate::output::Output;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

mod cfg;
use self::cfg::ControlFlowGraph;

mod edge_relation;
use self::edge_relation::EdgeSubsetRelation;

mod live_regions;
use self::live_regions::LiveRegions;

mod worklist;
use self::worklist::WorkList;

crate fn edge(tables: &InternerTables, dump_enabled: bool, all_facts: AllFacts) -> Output {
    let live_regions = &LiveRegions::from(tables, &all_facts);

    do_computation::<EdgeSubsetRelation>(tables, live_regions, dump_enabled, &all_facts)
}

// Compute the DYING regions at each point. A region R is DYING at a
// point Q if (a) R is dead at Q and (b) one of the following holds:
//
// - There is some predecessor P where R is live
// - There is some predecessor P where R appears in an outlives relation
//
// The latter is a bit wacky. The problem is that there are regions
// that are local to a specific point. It may be worth removing these
// as a kind of pre-pass. These arise from things like
// `foo::<'3>(...)`, where `'3` doesn't then appear in any variables
// or anything.

trait SubsetRelation: Clone {
    fn empty(num_regions: usize) -> Self;
    fn kill_region(&mut self, r1: Region);
    fn insert_one(&mut self, r1: Region, r2: Region) -> bool; // true if changed

    // true if changed
    fn insert_all(&mut self, other: &Self, live_regions: &BTreeSet<Region>) -> bool;

    fn for_each_reachable(&self, r1: Region, op: impl FnMut(Region));
}

fn do_computation<SR: SubsetRelation>(
    tables: &InternerTables,
    live_regions: &LiveRegions,
    dump_enabled: bool,
    all_facts: &AllFacts,
) -> Output {
    let cfg = &ControlFlowGraph::new(tables, all_facts);

    let subset =
        compute_subset::<EdgeSubsetRelation>(tables, live_regions, cfg, dump_enabled, &all_facts);

    let mut output = Output::new(dump_enabled);

    for point in tables.each::<Point>() {
        for region in tables.each::<Region>() {
            subset[point.index()].for_each_reachable(region, |successor| {
                output
                    .subset
                    .entry(point)
                    .or_insert(BTreeMap::default())
                    .entry(region)
                    .or_insert(BTreeSet::default())
                    .insert(successor);
            });
        }
    }

    output
}

fn compute_subset<SR: SubsetRelation>(
    tables: &InternerTables,
    live_regions: &LiveRegions,
    cfg: &ControlFlowGraph,
    _dump_enabled: bool,
    all_facts: &AllFacts,
) -> Vec<Rc<SR>> {
    let num_points = tables.len::<Point>();
    let num_regions = tables.len::<Region>();
    let mut relations_per_point: Vec<Option<Rc<SR>>> = (0..num_points).map(|_| None).collect();

    // Option 1:
    // - insert outlives once and never again
    // - iterate over CFG_EDGE and copy edges from pred to succ
    //
    // But:
    // - I wanted to have sharing between edges
    //
    // Alternative:
    // - start out as None
    // - when P -> Q is dirty:
    //   - load value from P, remove dead edges yield P1
    //   - if Q is None, store P1
    //   - if Q is Some, add P1 into it then drop

    let mut worklist = WorkList::new();

    // Pass 0. Initialize entry points to an empty subset.
    let entry_points: Vec<Point> = tables
        .each::<Point>()
        .filter(|&p| !cfg.has_predecessors(p))
        .collect();
    let empty = Rc::new(SR::empty(num_regions));
    for &point in &entry_points {
        assert!(relations_per_point[point.index()].is_none());
        relations_per_point[point.index()] = Some(empty.clone());
        worklist.add(point);
    }

    // Pass 1: insert the OUTLIVES relations into each point
    for (r1, r2, p) in &all_facts.outlives {
        let mut rpp = &mut relations_per_point[p.index()];
        let mut subsets = rpp.take()
            .unwrap_or(Rc::new(SubsetRelation::empty(num_regions)));
        Rc::make_mut(&mut subsets).insert_one(*r1, *r2);
        *rpp = Some(subsets);
        worklist.add(*p);
    }

    // Pass 2: propagate across cfg edges
    while let Some(p) = worklist.next() {
        // For each edge P -> Q:
        for q in cfg.successors(p) {
            // Find the subset relations on exit from P. This node is on
            // the worklist because some of these relations have not yet
            // been propagated to the successors of P.
            let mut rpp_p = relations_per_point[p.index()].clone().unwrap();

            // Remove any relations that are no live in Q.
            //
            // FIXME. This is probably not as efficient as it could
            // be. For example, if there are multiple successors,
            // likely there will be regions that are dead on *all* of
            // them, and that work is repeated.
            for r in live_regions.dying_on_edge(p, q) {
                Rc::make_mut(&mut rpp_p).kill_region(r);
            }

            let mut rpp_q_slot = &mut relations_per_point[q.index()];
            let q_changed = match rpp_q_slot.take() {
                None => {
                    // There was no previous value at `q`. Just use the
                    // value from `p` then.
                    *rpp_q_slot = Some(rpp_p);
                    true
                }

                Some(mut rpp_q) => {
                    // There was a previous value at `q`; so add the
                    // remaining regions from `rpp_p` into it. There
                    // may or may not be new things here.
                    let live_regions_at_p = live_regions.live_regions_at(p);
                    let changed = Rc::make_mut(&mut rpp_q).insert_all(&rpp_p, live_regions_at_p);
                    *rpp_q_slot = Some(rpp_q);
                    changed
                }
            };

            if q_changed {
                worklist.add(q);
            }
        }
    }

    relations_per_point
        .into_iter()
        .map(|r| r.unwrap_or_else(|| empty.clone()))
        .collect()
}
