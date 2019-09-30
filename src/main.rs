use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name="rsat", about=env!("CARGO_PKG_DESCRIPTION"), version=env!("CARGO_PKG_VERSION"),
setting=structopt::clap::AppSettings::ColoredHelp)]
struct Opt {
    #[structopt(parse(from_os_str), help = "Input file in DIMACS format")]
    file: PathBuf,
    #[structopt(
        short,
        long,
        default_value = "1",
        help = "Algorithm to use (1 -> CDCL, 2 -> SLS)"
    )]
    alg: u32,
    #[structopt(
        long = "max-tries",
        default_value = "100",
        help = "Maximum number of tries for SLS"
    )]
    max_tries: u32,
    #[structopt(
        long = "max-flips",
        default_value = "1000",
        help = "Maxinum number of flips in each try of SLS"
    )]
    max_flips: u32,
}

fn main() {
    let opt = Opt::from_args();
    let mut formula = rsat::sls::Formula::new_from_file(opt.file.to_str().unwrap());

    use rsat::common::Solution::*;
    let solution = match opt.alg {
        1 => {
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
        2 => formula.local_search(opt.max_tries, opt.max_flips, rsat::sls::ScoreFnType::Exp),
        _ => panic!("Invalid algorithm"),
    };
    match solution {
        Unsat => println!("UNSAT"),
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
