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
}

fn main() {
    let opt = Opt::from_args();
    let mut formula = rsat::sls::Formula::new_from_file(opt.file.to_str().unwrap()).unwrap();

    use rsat::Solution::*;
    let solution = match opt.alg {
        1 => {
            if opt.parallel {
                panic!("Parallelism is not implemented for CDCL solver yet.");
            }
            let mut solver = rsat::msat::Solver::new();
            for _ in 0..formula.n_vars() {
                solver.new_var();
            }
            for i in 0..formula.n_clauses() {
                let c = formula.ith_clause(i);
                let r = solver.new_clause(c.lits.clone());
                if !r {
                    println!("UNSAT");
                    return;
                }
            }
            solver.solve(vec![])
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
        Unsat => println!("UNSAT"),
        Unknown => println!("UNKNOWN"),
        Best(solution) => {
            println!("UNKNOWN");
            let solution = solution.iter().map(|&x| if x { 1 } else { -1 });
            for (i, v) in solution.enumerate() {
                print!("{} ", v * ((i + 1) as i32));
            }
            println!("0");
        }
        Sat(solution) => {
            if !formula.verify(&solution) {
                panic!("Solver gave incorrect model");
            }
            println!("SAT");
            let solution = solution.iter().map(|&x| if x { 1 } else { -1 });
            for (i, v) in solution.enumerate() {
                print!("{} ", v * ((i + 1) as i32));
            }
            println!("0");
        }
    }
}
