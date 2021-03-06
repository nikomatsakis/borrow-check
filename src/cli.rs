#![allow(deprecated)] // arg_enum! uses deprecated stuff

use crate::dump;
use crate::facts::{Loan, Point, Region};
use crate::intern;
use crate::output::Output;
use crate::tab_delim;
use failure::Error;
use std::path::Path;
use std::time::{Duration, Instant};
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum Algorithm {
        Naive,
        DatafrogOpt,
        LocationInsensitive,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "borrow-check")]
pub struct Opt {
    #[structopt(
        short = "a",
        default_value = "naive",
        raw(possible_values = "&Algorithm::variants()", case_insensitive = "true")
    )]
    algorithm: Algorithm,
    #[structopt(long = "skip-tuples")]
    skip_tuples: bool,
    #[structopt(long = "skip-timing")]
    skip_timing: bool,
    #[structopt(short = "v")]
    verbose: bool,
    #[structopt(short = "o", long = "output")]
    output_directory: Option<String>,
    #[structopt(raw(required = "true"))]
    fact_dirs: Vec<String>,
}

pub fn main(opt: Opt) -> Result<(), Error> {
    do catch {
        let output_directory = opt.output_directory.map(|x| Path::new(&x).to_owned());
        for facts_dir in opt.fact_dirs {
            let tables = &mut intern::InternerTables::new();

            let result: Result<(Duration, Output<Region, Loan, Point>), Error> = do catch {
                let verbose = opt.verbose;
                let algorithm = opt.algorithm;
                let all_facts =
                    tab_delim::load_tab_delimited_facts(tables, &Path::new(&facts_dir))?;
                timed(|| Output::compute(&all_facts, algorithm, verbose))
            };

            match result {
                Ok((duration, output)) => {
                    println!("--------------------------------------------------");
                    println!("Directory: {}", facts_dir);
                    if !opt.skip_timing {
                        let seconds: f64 = duration.as_secs() as f64;
                        let millis: f64 = duration.subsec_nanos() as f64 * 0.000_000_001_f64;
                        println!("Time: {:0.3}s", seconds + millis);
                    }
                    if !opt.skip_tuples {
                        dump::dump_output(&output, &output_directory, tables)
                            .expect("Failed to write output");
                    }
                }

                Err(error) => {
                    eprintln!("`{}`: {}", facts_dir, error);
                }
            }
        }
    }
}

fn timed<T>(op: impl FnOnce() -> T) -> (Duration, T) {
    let start = Instant::now();
    let output = op();
    let duration = start.elapsed();
    (duration, output)
}
