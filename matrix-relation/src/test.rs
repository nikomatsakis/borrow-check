#![cfg(test)]

use crate::Relation;

type TestRelation = Relation<usize>;

fn test(relation: &TestRelation, expected_lines: &[&str]) {
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
    let mut r = TestRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);

    test(&r, &["0 --> 1", "1 --> 2"]);
}

#[test]
fn add_remove_1() {
    let mut r = TestRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.kill(&[0, 2], &[1]);

    test(&r, &["0 --> 2"]);
}

#[test]
fn add_remove_0() {
    let mut r = TestRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.kill(&[1, 2], &[0]);

    test(&r, &["1 --> 2"]);
}

#[test]
fn add_remove_2() {
    let mut r = TestRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.kill(&[0, 1], &[2]);

    test(&r, &["0 --> 1"]);
}

#[test]
fn add_cycle() {
    let mut r = TestRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 0);

    test(&r, &["0 --> 1", "1 --> 2", "2 --> 0"]);
}

#[test]
fn remove_all() {
    let mut r = TestRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.kill(&[0, 2], &[1]);
    test(&r, &["0 --> 2"]);

    r.kill(&[0, 1], &[2]);
    test(&r, &[]);
}

#[test]
fn add_remove_cycle() {
    let mut r = TestRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 0);
    r.kill(&[0, 2], &[1]);

    test(&r, &["0 --> 2", "2 --> 0"]);
}

#[test]
fn remove_all_cycle() {
    let mut r = TestRelation::new(3);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 0);
    r.kill(&[0, 2], &[1]);

    test(&r, &["0 --> 2", "2 --> 0"]);

    r.kill(&[1, 2], &[0]);
    test(&r, &["2 --> 2"]);
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
    let mut r = TestRelation::new(5);

    r.add_edge(0, 2);
    r.add_edge(1, 2);
    r.add_edge(4, 2);
    r.add_edge(2, 3);

    r.kill(&[0, 1, 3, 4], &[2]);

    test(&r, &["0 --> 3", "1 --> 3", "4 --> 3"]);
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
    let mut r = TestRelation::new(6);

    r.add_edge(0, 2);
    r.add_edge(1, 2);
    r.add_edge(4, 2);
    r.add_edge(2, 3);
    r.add_edge(5, 3);

    r.kill(&[0, 1, 3, 4, 5], &[2]);

    test(
        &r,
        &["0 --> 3", "1 --> 3", "4 --> 3", "5 --> 3"],
    );
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
    let mut r = TestRelation::new(4);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(1, 3);

    test(&r, &["0 --> 1", "1 --> 2", "1 --> 3"]);

    r.kill(&[0, 2, 3], &[1]);

    test(&r, &["0 --> 2", "0 --> 3"]);
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
    let mut r = TestRelation::new(4);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(3, 2);

    r.kill(&[0, 2, 3], &[1]);

    test(&r, &["0 --> 2", "3 --> 2"]);
}

#[test]
fn long_remove_cycle() {
    let mut r = TestRelation::new(6);

    r.add_edge(0, 1);
    r.add_edge(1, 2);
    r.add_edge(2, 3);
    r.add_edge(3, 4);
    r.add_edge(4, 0);

    test(&r, &["0 --> 1", "1 --> 2", "2 --> 3", "3 --> 4", "4 --> 0"]);

    r.kill(&[0, 2, 3, 4], &[1]);
    test(
        &r,
        &["0 --> 2", "2 --> 3", "3 --> 4", "4 --> 0"],
    );

    r.kill(&[0, 1, 2, 4], &[3]);
    test(
        &r,
        &[
            "0 --> 2",
            "2 --> 4",
            "4 --> 0",
        ],
    );

    r.kill(&[1, 2, 3, 4], &[0]);
    test(
        &r,
        &[
            "2 --> 4",
            "4 --> 2",
        ],
    );
}

#[test]
fn multi_in_multi_out() {
    let mut r = TestRelation::new(5);

    r.add_edge(0, 2);
    r.add_edge(1, 2);
    r.add_edge(2, 3);
    r.add_edge(2, 4);
    test(&r, &["0 --> 2", "1 --> 2", "2 --> 3", "2 --> 4"]);

    r.kill(&[0, 1, 3, 4], &[2]);
    test(&r, &["0 --> 3", "0 --> 4", "1 --> 3", "1 --> 4"]);
}
