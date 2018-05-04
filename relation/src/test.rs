#![cfg(test)]

use crate::indices::NodeIndex;
use crate::vec_family::{StdVec, VecFamily};
use crate::Relation;

type StdVecRelation = Relation<StdVec>;

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
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let mut r = StdVecRelation::new(3);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);

    test(&r, &["N(0) --E(0)--> N(1)", "N(1) --E(1)--> N(2)"]);
}

#[test]
fn add_remove_1() {
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let mut r = StdVecRelation::new(3);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.remove_edges(n1);

    test(&r, &["N(0) --E(0)--> N(2)", "free edge E(1)"]);
}

#[test]
fn add_remove_0() {
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let mut r = StdVecRelation::new(3);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.remove_edges(n0);

    test(&r, &["N(1) --E(1)--> N(2)", "free edge E(0)"]);
}

#[test]
fn add_remove_2() {
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let mut r = StdVecRelation::new(3);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.remove_edges(n2);

    test(&r, &["N(0) --E(0)--> N(1)", "free edge E(1)"]);
}

#[test]
fn add_cycle() {
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let mut r = StdVecRelation::new(3);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.add_edge(n2, n0);

    test(&r, &["N(0) --E(0)--> N(1)",
               "N(1) --E(1)--> N(2)",
               "N(2) --E(2)--> N(0)",
              ]);
}

// FIXME(chrisvittal) removing all edges of a graph panics
#[test]
fn remove_all() {
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let mut r = StdVecRelation::new(3);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.remove_edges(n1);

    test(&r, &["N(0) --E(0)--> N(2)", "free edge E(1)"]);

    r.remove_edges(n1);
    test(&r, &["free edge E(0)", "free edge E(1)"]);
}

#[test]
fn add_remove_cycle() {
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let mut r = StdVecRelation::new(3);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.add_edge(n2, n0);
    r.remove_edges(n1);

    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(0)",
               "free edge E(1)",
              ]);
}

// FIXME(chrisvittal) removing all edges of a cylce graph panics
// differently than just removing all edges
#[test]
fn remove_all_cycle() {
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let mut r = StdVecRelation::new(3);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.add_edge(n2, n0);
    r.remove_edges(n1);
    r.remove_edges(n0);

    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(0)",
               "free edge E(1)",
              ]);
}

#[test]
fn add_remove_cycle_out() {
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let mut r = StdVecRelation::new(3);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.add_edge(n2, n0);
    println!("{:#?}", r);
    r.remove_edges(n1);
    println!("{:#?}", r);
    r.remove_edges(n0);
    println!("{:#?}", r);
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
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let n3: NodeIndex = NodeIndex::from(3);
    let n4: NodeIndex = NodeIndex::from(4);
    let mut r = StdVecRelation::new(5);

    r.add_edge(n0, n2);
    r.add_edge(n1, n2);
    r.add_edge(n4, n2);
    r.add_edge(n2, n3);

    r.remove_edges(n2);
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
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let n3: NodeIndex = NodeIndex::from(3);
    let n4: NodeIndex = NodeIndex::from(4);
    let n5: NodeIndex = NodeIndex::from(5);
    let mut r = StdVecRelation::new(6);

    r.add_edge(n0, n2);
    r.add_edge(n1, n2);
    r.add_edge(n4, n2);
    r.add_edge(n2, n3);
    r.add_edge(n5, n3);

    r.remove_edges(n2);
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
// This test exercieses unimplemented functionality.
// as such, it is disabled
#[ignore]
fn remove_one_incoming_two_outgoing() {
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let n3: NodeIndex = NodeIndex::from(3);
    let mut r = StdVecRelation::new(4);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.add_edge(n1, n3);

    println!("{:#?}", r);

    r.remove_edges(n1);
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
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let n3: NodeIndex = NodeIndex::from(3);

    let mut r = StdVecRelation::new(4);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.add_edge(n3, n2);

    r.remove_edges(n1);

    test(&r, &["N(0) --E(0)--> N(2)",
               "N(3) --E(2)--> N(2)",
               "free edge E(1)",
              ]);
}

// FIXME(chrisvittal) removing all edges of a cylce graph panics
// This one even panics before trying to remove all edges
#[test]
fn long_remove_cycle() {
    let n0: NodeIndex = NodeIndex::from(0);
    let n1: NodeIndex = NodeIndex::from(1);
    let n2: NodeIndex = NodeIndex::from(2);
    let n3: NodeIndex = NodeIndex::from(3);
    let n4: NodeIndex = NodeIndex::from(4);

    let mut r = StdVecRelation::new(6);

    r.add_edge(n0, n1);
    r.add_edge(n1, n2);
    r.add_edge(n2, n3);
    r.add_edge(n3, n4);
    r.add_edge(n4, n0);

    test(&r, &["N(0) --E(0)--> N(1)",
               "N(1) --E(1)--> N(2)",
               "N(2) --E(2)--> N(3)",
               "N(3) --E(3)--> N(4)",
               "N(4) --E(4)--> N(0)",
              ]);

    r.remove_edges(n1);
    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(3)",
               "N(3) --E(3)--> N(4)",
               "N(4) --E(4)--> N(0)",
               "free edge E(1)",
              ]);

    r.remove_edges(n3);
    test(&r, &["N(0) --E(0)--> N(2)",
               "N(2) --E(2)--> N(4)",
               "N(4) --E(4)--> N(0)",
               "free edge E(3)",
               "free edge E(1)",
              ]);

    r.remove_edges(n0);
    test(&r, &["N(2) --E(2)--> N(4)",
               "N(4) --E(4)--> N(2)",
               "free edge E(0)",
               "free edge E(3)",
               "free edge E(1)",
              ]);
}
