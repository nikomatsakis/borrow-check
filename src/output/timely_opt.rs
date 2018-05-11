// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Timely dataflow runs on its own thread.

use crate::facts::{AllFacts, Point};
use crate::output::Output;
use differential_dataflow::collection::Collection;
use differential_dataflow::operators::iterate::Variable;
use differential_dataflow::operators::*;
use std::collections::{BTreeMap, BTreeSet};
use std::mem;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use timely;
use timely::dataflow::operators::*;
use timely::dataflow::Scope;

pub(super) fn compute(dump_enabled: bool, mut all_facts: AllFacts) -> Output {
    // Declare that each universal region is live at every point.
    let all_points: BTreeSet<Point> = all_facts
        .cfg_edge
        .iter()
        .map(|&(p, _)| p)
        .chain(all_facts.cfg_edge.iter().map(|&(_, q)| q))
        .collect();

    for &r in &all_facts.universal_region {
        for &p in &all_points {
            all_facts.region_live_at.push((r, p));
        }
    }

    let result = Arc::new(Mutex::new(Output::new(dump_enabled)));

    // Use a channel to send `all_facts` to one worker (and only one)
    let (tx, rx) = mpsc::channel();
    tx.send(all_facts).unwrap();
    mem::drop(tx);
    let rx = Mutex::new(rx);

    timely::execute_from_args(vec![].into_iter(), {
        let result = result.clone();
        move |worker| {
            // First come, first serve: one worker gets all the facts;
            // the others get empty vectors.
            let my_facts = rx.lock()
                .unwrap()
                .recv()
                .unwrap_or_else(|_| AllFacts::default());

            worker.dataflow::<(), _, _>(|scope| {
                macro_rules! let_collections {
                    (let ($($facts:ident,)*) = ..$base:expr;) => {
                        let ($($facts),*) = (
                            $(Collection::<_, _, isize>::new(
                                $base.$facts
                                    .to_stream(scope)
                                    .map(|datum| (datum, Default::default(), 1)),
                            ),)*
                        );
                    }
                }

                let_collections! {
                    let (
                        borrow_region,
                        cfg_edge,
                        killed,
                        outlives,
                        region_live_at,
                    ) = ..my_facts;
                }

                // .decl subset(R1, R2, P)
                //
                // At the point P, R1 <= R2.
                let subset = scope.scoped(|subset_scope| {
                    let outlives = outlives.enter(&subset_scope);
                    let cfg_edge = cfg_edge.enter(&subset_scope);
                    let region_live_at = region_live_at.enter(&subset_scope);

                    // subset(R1, R2, P) :- outlives(R1, R2, P).
                    let subset_base = outlives.clone();
                    let subset = Variable::from(subset_base.clone());

                    // .decl live_to_dead_regions(R1, R2, P, Q)
                    //
                    // The regions `R1` and `R2` are "live to dead"
                    // on the edge `P -> Q` if:
                    //
                    // - In P, `R1` <= `R2`
                    // - In Q, `R1` is live but `R2` is dead.
                    //
                    // In that case, `Q` would like to add all the
                    // live things reachable from `R2` to `R1`.
                    //
                    // live_to_dead_regions(R1, R2, P, Q) :-
                    //   subset(R1, R2, P),
                    //   cfg_edge(P, Q),
                    //   region_live_at(R1, Q),
                    //   !region_live_at(R2, Q).
                    let live_to_dead_regions = {
                        subset
                            .map(|(r1, r2, p)| (p, (r1, r2)))
                            .join(&cfg_edge)
                            .map(|(p, (r1, r2), q)| ((r1, q), (r2, p)))
                            .semijoin(&region_live_at)
                            .map(|((r1, q), (r2, p))| ((r2, q), (r1, p)))
                            .antijoin(&region_live_at)
                            .map(|((r2, q), (r1, p))| (r1, r2, p, q))
                    };

                    // .decl dead_can_reach(R1, R2, P, Q)
                    //
                    // Indicates that the region `R1`, which is dead
                    // in `Q`, can reach the region `R2` in P.
                    //
                    // This is effectively the transitive subset
                    // relation, but we try to limit it to regions
                    // that are dying on the edge P -> Q.
                    let dead_can_reach = {
                        // dead_can_reach(R2, R3, P, Q) :-
                        //   live_to_dead_regions(_R1, R2, P, Q),
                        //   subset(R2, R3, P).
                        let dead_can_reach_base = {
                            live_to_dead_regions
                                .map(|(_r1, r2, p, q)| ((r2, p), q))
                                .join(&subset.map(|(r2, r3, p)| ((r2, p), r3)))
                                .map(|((r2, p), q, r3)| (r2, r3, p, q))
                        };

                        let dead_can_reach = Variable::from(dead_can_reach_base.clone());

                        // dead_can_reach(R1, R3, P, Q) :-
                        //   dead_can_reach(R1, R2, P, Q),
                        //   !region_live_at(R2, Q),
                        //   subset(R2, R3, P).
                        //
                        // This is the "transitive closure" rule, but
                        // note that we only apply it with the
                        // "intermediate" region R2 is dead at Q.
                        let dead_can_reach2 = {
                            dead_can_reach
                                .map(|(r1, r2, p, q)| ((r2, q), (r1, p)))
                                .antijoin(&region_live_at)
                                .map(|((r2, q), (r1, p))| ((r2, p), (r1, q)))
                                .join(&subset.map(|(r2, r3, p)| ((r2, p), r3)))
                                .map(|((_r2, p), (r1, q), r3)| (r1, r3, p, q))
                        };

                        dead_can_reach.set(&dead_can_reach_base.concat(&dead_can_reach2).distinct())
                    };

                    // subset(R1, R2, Q) :-
                    //   subset(R1, R2, P) :-
                    //   cfg_edge(P, Q),
                    //   region_live_at(R1, Q),
                    //   region_live_at(R2, Q).
                    //
                    // Carry `R1 <= R2` from P into Q if both `R1` and
                    // `R2` are live in Q.
                    let subset1 = subset
                        .map(|(r1, r2, p)| (p, (r1, r2)))
                        .join(&cfg_edge)
                        .map(|(_p, (r1, r2), q)| ((r1, q), r2))
                        .semijoin(&region_live_at)
                        .map(|((r1, q), r2)| ((r2, q), r1))
                        .semijoin(&region_live_at)
                        .map(|((r2, q), r1)| (r1, r2, q));

                    // subset(R1, R3, Q) :-
                    //   live_to_dead_regions(R1, R2, P, Q),
                    //   dead_can_reach(R2, R3, P, Q),
                    //   region_live_at(R3, Q).
                    let subset2 = {
                        live_to_dead_regions
                            .map(|(r1, r2, p, q)| ((r2, p, q), r1))
                            .join(&dead_can_reach.map(|(r2, r3, p, q)| ((r2, p, q), r3)))
                            .map(|((_r2, _p, q), r1, r3)| ((r3, q), r1))
                            .semijoin(&region_live_at)
                            .map(|((r3, q), r1)| (r1, r3, q))
                    };

                    subset
                        .set(&subset_base.concat(&subset1).concat(&subset2).distinct())
                        .leave()
                });

                // .decl requires(R, B, P) -- at the point, things with region R
                // may depend on data from borrow B
                let requires = borrow_region.iterate(|requires| {
                    let borrow_region = borrow_region.enter(&requires.scope());
                    let subset = subset.enter(&requires.scope());
                    let killed = killed.enter(&requires.scope());
                    let region_live_at = region_live_at.enter(&requires.scope());
                    let cfg_edge = cfg_edge.enter(&requires.scope());

                    // requires(R, B, P) :- borrow_region(R, B, P).
                    let requires1 = borrow_region.clone();

                    // requires(R2, B, P) :-
                    //   requires(R1, B, P),
                    //   subset(R1, R2, P).
                    let requires2 = requires
                        .map(|(r1, b, p)| ((r1, p), b))
                        .join(&subset.map(|(r1, r2, p)| ((r1, p), r2)))
                        .map(|((_r1, p), b, r2)| (r2, b, p));

                    // requires(R, B, Q) :-
                    //   requires(R, B, P),
                    //   !killed(B, P),
                    //   cfg_edge(P, Q),
                    //   region_live_at(R, Q).
                    let requires3 = requires
                        .map(|(r, b, p)| ((b, p), r))
                        .antijoin(&killed)
                        .map(|((b, p), r)| (p, (r, b)))
                        .join(&cfg_edge)
                        .map(|(_p, (r, b), q)| ((r, q), b))
                        .semijoin(&region_live_at)
                        .map(|((r, q), b)| (r, b, q));

                    requires1
                        .concat(&requires2)
                        .concat(&requires3)
                        .distinct()
                });

                // .decl borrow_live_at(B, P) -- true if the restrictions of the borrow B
                // need to be enforced at the point P
                let borrow_live_at = {
                    // borrow_live_at(B, P) :- requires(R, B, P), region_live_at(R, P)
                    let borrow_live_at1 = requires
                        .map(|(r, b, p)| ((r, p), b))
                        .semijoin(&region_live_at)
                        .map(|((_r, p), b)| (b, p));

                    borrow_live_at1.distinct()
                };

                if dump_enabled {
                    region_live_at.inspect_batch({
                        let result = result.clone();
                        move |_timestamp, facts| {
                            let mut result = result.lock().unwrap();
                            for ((region, location), _timestamp, multiplicity) in facts {
                                assert_eq!(*multiplicity, 1);
                                result
                                    .region_live_at
                                    .entry(*location)
                                    .or_insert(vec![])
                                    .push(*region);
                            }
                        }
                    });

                    subset.inspect_batch({
                        let result = result.clone();
                        move |_timestamp, facts| {
                            let mut result = result.lock().unwrap();
                            for ((r1, r2, location), _timestamp, multiplicity) in facts {
                                assert_eq!(*multiplicity, 1);
                                result
                                    .subset
                                    .entry(*location)
                                    .or_insert(BTreeMap::new())
                                    .entry(*r1)
                                    .or_insert(BTreeSet::new())
                                    .insert(*r2);
                                result.region_degrees.update_degrees(*r1, *r2, *location);
                            }
                        }
                    });

                    requires.inspect_batch({
                        let result = result.clone();
                        move |_timestamp, facts| {
                            let mut result = result.lock().unwrap();
                            for ((region, borrow, location), _timestamp, multiplicity) in facts {
                                assert_eq!(*multiplicity, 1);
                                result
                                    .restricts
                                    .entry(*location)
                                    .or_insert(BTreeMap::new())
                                    .entry(*region)
                                    .or_insert(BTreeSet::new())
                                    .insert(*borrow);
                            }
                        }
                    });
                }

                borrow_live_at.inspect_batch({
                    let result = result.clone();
                    move |_timestamp, facts| {
                        let mut result = result.lock().unwrap();
                        for ((borrow, location), _timestamp, multiplicity) in facts {
                            assert_eq!(*multiplicity, 1);
                            result
                                .borrow_live_at
                                .entry(*location)
                                .or_insert(Vec::new())
                                .push(*borrow);
                        }
                    }
                });
            });
        }
    }).unwrap();

    // Remove from the Arc and Mutex, since it is fully populated now.
    Arc::try_unwrap(result)
        .unwrap_or_else(|_| panic!("somebody still has a handle to this arc"))
        .into_inner()
        .unwrap()
}