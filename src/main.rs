use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name="rsat", about=env!("CARGO_PKG_DESCRIPTION"), version=env!("CARGO_PKG_VERSION"))]
struct Opt {
    #[structopt(parse(from_os_str), help = "Input file")]
    file: PathBuf,
    #[structopt(
        long = "max-tries",
        default_value = "100",
        help = "Maximum number of tries"
    )]
    max_tries: u32,
    #[structopt(
        long = "max-flips",
        default_value = "1000",
        help = "Maxinum number of flips in each try"
    )]
    max_flips: u32,
}

fn main() {
    let opt = Opt::from_args();
    let mut formula = rsat::sls::Formula::new_from_file(opt.file.to_str().unwrap());

    use rsat::common::Solution::*;
    match formula.local_search(opt.max_tries, opt.max_flips) {
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
