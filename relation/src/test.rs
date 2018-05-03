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
