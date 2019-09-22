use clap::{App, Arg};

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let matches = App::new("rsat")
        .version(version)
        .about("SolHOP SAT Solver")
        .arg(
            Arg::with_name("file")
                .index(1)
                .required(true)
                .help("Input file"),
        )
        .arg(
            Arg::with_name("max-tries")
                .long("max-tries")
                .takes_value(true)
                .help("Maximum number of tries"),
        )
        .arg(
            Arg::with_name("max-flips")
                .long("max-flips")
                .takes_value(true)
                .help("Maxinum number of flips in each try"),
        )
        .get_matches();
    let mut formula = match matches.value_of("file") {
        None => panic!("File name is required"),
        Some(input_file) => rsat::sls::Formula::new_from_file(input_file),
    };
    let max_tries = match matches.value_of("max-tries") {
        None => 100,
        Some(n) => n.parse().expect("Expected an integer"),
    };
    let max_flips = match matches.value_of("max-flips") {
        None => 1000,
        Some(n) => n.parse().expect("Expected an integer"),
    };

    use rsat::common::Solution::*;
    match formula.local_search(max_tries, max_flips) {
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
