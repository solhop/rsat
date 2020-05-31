use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name="rsat", about=env!("CARGO_PKG_DESCRIPTION"), version=env!("CARGO_PKG_VERSION"),
setting=structopt::clap::AppSettings::ColoredHelp)]
struct Opt {
    /// Input file in DIMACS format
    #[structopt(parse(from_os_str))]
    file: PathBuf,
    /// Algorithm to use (1 -> CDCL, 2 -> SLS)
    #[structopt(short, long, default_value = "1")]
    alg: u32,
    /// Enables data parallelism (currently only for sls solver)
    #[structopt(short, long)]
    parallel: bool,
    /// Maximum number of tries for SLS
    #[structopt(long = "max-tries", default_value = "100")]
    max_tries: u32,
    /// Maxinum number of flips in each try of SLS
    #[structopt(long = "max-flips", default_value = "1000")]
    max_flips: u32,
    /// Drat file to log conflict clauses addition and deletion
    #[structopt(long, parse(from_os_str))]
    drat: Option<PathBuf>,
}

// Function to write drat clauses to file
fn write_drat_clauses(drat: Option<File>, solver: rsat::cdcl::Solver) {
    if let Some(mut drat_file) = drat {
        for (lits, is_delete) in solver.drat_clauses() {
            if is_delete {
                write!(drat_file, "d ").unwrap();
            }
            for lit in lits.iter() {
                write!(
                    drat_file,
                    "{} ",
                    if lit.sign() {
                        -(lit.var().index() as i32 + 1)
                    } else {
                        lit.var().index() as i32 + 1
                    }
                )
                .unwrap();
            }
            writeln!(drat_file, "0").unwrap();
        }
    }
}

fn main() {
    let opt = Opt::from_args();
    let mut formula = rsat::sls::Solver::new_from_file(opt.file.to_str().unwrap());
    let drat = match opt.drat {
        Some(drat) => Some(File::create(drat).expect("Drat file not found")),
        None => None,
    };

    use rsat::Solution::*;
    let solution = match opt.alg {
        1 => {
            if opt.parallel {
                panic!("Parallelism is not implemented for CDCL solver yet.");
            }

            use rsat::cdcl::*;

            let mut options = SolverOptions::default();
            // options.branching_heuristic = BranchingHeuristic::Vsids {
            //     var_inc: 1.0,
            //     var_decay: 0.95,
            // };
            if drat.is_some() {
                options.capture_drat = true;
            }
            let mut solver = Solver::new(options);

            for _ in 0..formula.n_vars() {
                solver.new_var();
            }

            for i in 0..formula.n_clauses() {
                let c = formula.ith_clause(i);
                solver.add_clause(c.lits.clone());
            }

            let solution = solver.solve(vec![]);

            if let Unsat = solution {
                write_drat_clauses(drat, solver);
            }
            solution
        }
        2 => formula.local_search(
            opt.max_tries,
            opt.max_flips,
            rsat::sls::ScoreFnType::Exp,
            opt.parallel,
        ),
        _ => panic!("Invalid algorithm"),
    };
    match solution {
        Unsat => println!("s UNSATISFIABLE"),
        Unknown => println!("s UNKNOWN"),
        Best(solution) => {
            println!("s UNKNOWN");
            let solution = solution.iter().map(|&x| if x { 1 } else { -1 });
            print!("v ");
            for (i, v) in solution.enumerate() {
                print!("{} ", v * ((i + 1) as i32));
            }
            println!("0");
        }
        Sat(solution) => {
            if !formula.verify(&solution) {
                panic!("Solver gave incorrect model");
            }
            println!("s SATISFIABLE");
            print!("v ");
            let solution = solution.iter().map(|&x| if x { 1 } else { -1 });
            for (i, v) in solution.enumerate() {
                print!("{} ", v * ((i + 1) as i32));
            }
            println!("0");
        }
    }
}
