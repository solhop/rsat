use rsat::cdcl;
use rsat::common::{Solution, Var};
use std::fs::File;
use std::io;
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

fn parse_from_file(filename: &str) -> (usize, Vec<Vec<i32>>) {
    let file = File::open(filename).expect("File not found");
    let mut reader = io::BufReader::new(file);
    let parsed = rsat::parser::parse_dimacs_from_buf_reader(&mut reader);
    if let rsat::parser::Dimacs::Cnf { n_vars, clauses } = parsed {
        (n_vars, clauses)
    } else {
        panic!("Incorrect input format");
    }
}

// Function to write drat clauses to file
fn write_drat_clauses(drat: Option<File>, solver: rsat::cdcl::Solver) {
    use cdcl::DratClause;
    if let Some(mut drat_file) = drat {
        if let Some(drat_clauses) = solver.drat_clauses() {
            for drat_clause in drat_clauses {
                let (is_delete, lits) = match drat_clause {
                    DratClause::Add(lits) => (false, lits),
                    DratClause::Delete(lits) => (true, lits),
                };
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
}

fn main() {
    let opt = Opt::from_args();
    let (n_vars, clauses) = parse_from_file(opt.file.to_str().unwrap());

    let solution = match opt.alg {
        1 => {
            if opt.parallel {
                panic!("Parallelism is not implemented for CDCL solver yet.");
            }

            use cdcl::{Solver, SolverOptions};

            let mut options = SolverOptions::default();
            // options.branching_heuristic = BranchingHeuristic::Vsids {
            //     var_inc: 1.0,
            //     var_decay: 0.95,
            // };
            let drat = match opt.drat {
                Some(drat) => Some(File::create(drat).expect("Drat file not found")),
                None => None,
            };
            if drat.is_some() {
                options.capture_drat = true;
            }
            let mut solver = Solver::new(options);

            let vars: Vec<Var> = (0..n_vars).map(|_| solver.new_var()).collect();

            for clause in clauses {
                let lits = clause
                    .into_iter()
                    .map(|l| {
                        let var = vars[(l.abs() - 1) as usize];
                        if l < 0 {
                            var.neg_lit()
                        } else {
                            var.pos_lit()
                        }
                    })
                    .collect();
                solver.add_clause(lits);
            }

            let solution = solver.solve(vec![]);

            if let Solution::Unsat = solution {
                write_drat_clauses(drat, solver);
            }
            solution
        }
        2 => {
            let mut solver = rsat::sls::Solver::new_from_file(opt.file.to_str().unwrap());
            solver.local_search(
                opt.max_tries,
                opt.max_flips,
                rsat::sls::ScoreFnType::Exp,
                opt.parallel,
            )
        }
        _ => panic!("Invalid algorithm"),
    };
    match solution {
        Solution::Unsat => println!("s UNSATISFIABLE"),
        Solution::Unknown => println!("s UNKNOWN"),
        Solution::Best(solution) => {
            println!("s UNKNOWN");
            let solution = solution.iter().map(|&x| if x { 1 } else { -1 });
            print!("v ");
            for (i, v) in solution.enumerate() {
                print!("{} ", v * ((i + 1) as i32));
            }
            println!("0");
        }
        Solution::Sat(solution) => {
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
