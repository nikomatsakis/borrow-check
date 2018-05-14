#![cfg(test)]

use rand::{self, SeedableRng};
use rand::distributions::range::Range;
use rand::distributions::IndependentSample;

use crate::vec_family::{StdVec, VecFamily};
use crate::Relation;

type StdVecRelation = Relation<StdVec<usize>>;

fn test(relation: &Relation<impl VecFamily>, expected_lines: &[&str]) {
    let actual_lines = relation.dump_and_assert();

    for (expected_line, actual_line) in expected_lines.iter().zip(&actual_lines) {
        assert_eq!(
            expected_line, actual_line,
            "expected: {:#?}\nactual:\n{:#?}\n",
            expected_lines, actual_lines,
        );
    }

    assert_eq!(
        expected_lines.len(),
        actual_lines.len(),
        "expected: {:#?}\nactual:\n{:#?}\n",
        expected_lines,
        actual_lines,
    );
}

#[test]
fn add() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);

    test(&r, &["N(0) --E(0)--> N(1)", "N(1) --E(1)--> N(2)"]);
}

#[test]
fn add_remove_1() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.remove_edges(1);

    test(&r, &["N(0) --E(0)--> N(2)", "free edge E(1)"]);
}

#[test]
fn add_remove_0() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.remove_edges(0);

    test(&r, &["N(1) --E(1)--> N(2)", "free edge E(0)"]);
}

#[test]
fn add_remove_2() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.remove_edges(2);

    test(&r, &["N(0) --E(0)--> N(1)", "free edge E(1)"]);
}

#[test]
fn add_cycle() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 0);

    test(&r, &["N(0) --E(0)--> N(1)",
               "N(1) --E(1)--> N(2)",
               "N(2) --E(2)--> N(0)",
              ]);
}

#[test]
fn remove_all() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.remove_edges(1);

    test(&r, &["N(0) --E(0)--> N(2)", "free edge E(1)"]);
    r.remove_edges(2);
    test(&r, &["free edge E(0)", "free edge E(1)"]);
}

#[test]
fn add_remove_cycle() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 0);
    r.remove_edges(1);

    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(0)",
               "free edge E(1)",
              ]);
}

#[test]
fn remove_all_cycle() {
    let mut r = StdVecRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 0);
    r.remove_edges(1);

    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(0)",
               "free edge E(1)",
              ]);

    r.remove_edges(0);
    test(&r, &["N(2) --E(2)--> N(2)",
               "free edge E(0)",
               "free edge E(1)",
              ]);
}

// This test has a start graph
//
// 0 --> 2
// 1 --> 2
// 2 --> 3
//
// And wants an end graph
// 0 --> 3
// 1 --> 3
#[test]
fn remove_three_incoming_one_outgoing() {
    let mut r = StdVecRelation::new(5);

    r.add_edge(0, 2);
    r.add_edge(1, 2);
    r.add_edge(4, 2);
    r.add_edge(2, 3);

    r.remove_edges(2);
    test(&r, &["N(0) --E(0)--> N(3)",
               "N(1) --E(1)--> N(3)",
               "N(4) --E(2)--> N(3)",
               "free edge E(3)",
              ]);
}

// Start
// 0 --> 2
// 1 --> 2
// 2 --> 3
// 4 --> 2
// 5 --> 3
//
// End graph
// 0 --> 3
// 1 --> 3
// 4 --> 3
// 5 --> 3
#[test]
fn remove_three_incoming_one_outgoing_2() {
    let mut r = StdVecRelation::new(6);

    r.add_edge(0, 2);
    r.add_edge(1, 2);
    r.add_edge(4, 2);
    r.add_edge(2, 3);
    r.add_edge(5, 3);

    r.remove_edges(2);
    test(&r, &["N(0) --E(0)--> N(3)",
               "N(1) --E(1)--> N(3)",
               "N(4) --E(2)--> N(3)",
               "N(5) --E(4)--> N(3)",
               "free edge E(3)",
              ]);
}

// This test has a start graph
//
// 0 --> 1
// 1 --> 2
// 1 --> 3
//
// And wants an end graph
// 0 --> 2
// 0 --> 3
#[test]
fn remove_one_incoming_two_outgoing() {
    let mut r = StdVecRelation::new(4);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(1, 3);

    test(&r, &["N(0) --E(0)--> N(1)",
               "N(1) --E(2)--> N(3)",
               "N(1) --E(1)--> N(2)",
              ]);

    r.remove_edges(1);
    test(&r, &["N(0) --E(1)--> N(2)",
               "N(0) --E(2)--> N(3)",
               "free edge E(0)",
              ]);
}

// Graph From:
// 0 --> 1
// 1 --> 2
// 3 --> 2
//
// Graph To:
// 0 --> 2
// 3 --> 2
//
#[test]
fn add_remove_complex_1() {
    let mut r = StdVecRelation::new(4);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(3, 2);

    r.remove_edges(1);

    test(&r, &["N(0) --E(0)--> N(2)",
               "N(3) --E(2)--> N(2)",
               "free edge E(1)",
              ]);
}

#[test]
fn long_remove_cycle() {
    let mut r = StdVecRelation::new(6);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 3);
    r.add_edge(3, 4);
    r.add_edge(4, 0);

    test(&r, &["N(0) --E(0)--> N(1)",
               "N(1) --E(1)--> N(2)",
               "N(2) --E(2)--> N(3)",
               "N(3) --E(3)--> N(4)",
               "N(4) --E(4)--> N(0)",
              ]);

    r.remove_edges(1);
    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(3)",
               "N(3) --E(3)--> N(4)",
               "N(4) --E(4)--> N(0)",
               "free edge E(1)",
              ]);

    r.remove_edges(3);
    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(4)",
               "N(4) --E(4)--> N(0)",
               "free edge E(3)",
               "free edge E(1)",
              ]);

    r.remove_edges(0);
    test(&r, &["N(2) --E(2)--> N(4)",
               "N(4) --E(4)--> N(2)",
               "free edge E(0)",
               "free edge E(3)",
               "free edge E(1)",
              ]);
}

#[test]
fn multi_in_multi_out() {
    let mut r = StdVecRelation::new(5);

    r.add_edge(0, 2);
    r.add_edge(1, 2);
    r.add_edge(2, 3);
    r.add_edge(2, 4);
    test(&r, &["N(0) --E(0)--> N(2)",
               "N(1) --E(1)--> N(2)",
               "N(2) --E(3)--> N(4)",
               "N(2) --E(2)--> N(3)",
              ]);

    r.remove_edges(2);
    test(&r, &["N(0) --E(3)--> N(3)",
               "N(0) --E(1)--> N(4)",
               "N(1) --E(2)--> N(3)",
               "N(1) --E(0)--> N(4)",
              ]);
}

#[test]
fn scratch_random() {
    let mut r = StdVecRelation::new(1000);
    let range = Range::new(0, 1000);

    let mut rng = rand::StdRng::from_seed(&[1,2,3,4]);
    let rng = &mut rng;

    for _ in 0..300 {
        let (mut src, mut dst) = (range.ind_sample(rng), range.ind_sample(rng));
        while src == dst {
            src = range.ind_sample(rng);
            dst = range.ind_sample(rng);
        }
        println!("r.add_edge({}, {});", src, dst);
        r.add_edge(src, dst);
    }

    for i in 0..1000 {
        println!("r.remove_edges({});", i);
        r.remove_edges(i);
    }
}

#[test]
fn scratch_explicit() {
    // This case started by using the operations from above, and then
    // deleting rows that still made it fail, then renumbering.
    let mut r = StdVecRelation::new(5);
    r.add_edge(0, 1);
    r.add_edge(2, 3);
    r.add_edge(3, 4);
    r.add_edge(1, 4);
    r.remove_edges(1);
    r.remove_edges(3);
}
