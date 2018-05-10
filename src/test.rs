#![cfg(test)]

use crate::cli::Algorithm;
use crate::intern;
use crate::output::Output;
use crate::tab_delim;
use failure::Error;
use std::fmt::Debug;
use std::path::Path;

fn test_fn(dir_name: &str, fn_name: &str) -> Result<(), Error> {
    do catch {
        let facts_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("inputs")
            .join(dir_name)
            .join("nll-facts")
            .join(fn_name);
        println!("facts_dir = {:?}", facts_dir);
        let tables = &mut intern::InternerTables::new();
        let all_facts = tab_delim::load_tab_delimited_facts(tables, &facts_dir)?;

        // the naive algorithm is the "reference result"
        let naive_result = Output::compute(tables, all_facts.clone(), Algorithm::Naive, true);

        let bespoke_edge_result = Output::compute(tables, all_facts, Algorithm::BespokeEdge, true);

        compare(
            "bespoke-edge-subset",
            naive_result.subset(),
            bespoke_edge_result.subset(),
        );
    }
}

fn is_both<T>(m: &diff::Result<T>) -> bool {
    match m {
        diff::Result::Left(_) | diff::Result::Right(_) => false,
        diff::Result::Both(..) => true,
    }
}

fn compare(tag: &str, naive_value: impl Debug, other_value: impl Debug) {
    let naive_str = format!("{:#?}", naive_value);
    let other_str = format!("{:#?}", other_value);
    if naive_str == other_str {
        return;
    }

    println!("tag = {}", tag);

    let mut diffs = diff::lines(&naive_str, &other_str);

    // strip any trailing `Both` lines
    while is_both(diffs.last().unwrap()) {
        diffs.pop();
    }

    for m in diffs.into_iter().skip_while(|m| is_both(m)) {
        match m {
            diff::Result::Left(a) => println!("- {}", a),
            diff::Result::Right(a) => println!("+ {}", a),
            diff::Result::Both(a, _) => println!("  {}", a),
        }
    }
}

macro_rules! tests {
    ($($name:ident($dir:expr, $fn:expr),)*) => {
        $(
            #[test]
            fn $name() -> Result<(), Error> {
                test_fn($dir, $fn)
            }
        )*
    }
}

tests! {
    issue_47680("issue-47680", "main"),
}
