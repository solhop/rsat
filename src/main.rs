use clap::{App, Arg};

fn main() {
    let matches = App::new("rsat")
        .version("0.1.0")
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
        Some(input_file) => rsat::Formula::new_from_file(input_file),
    };
    let max_tries = match matches.value_of("max-tries") {
        None => 100,
        Some(n) => n.parse().expect("Expected an integer"),
    };
    let max_flips = match matches.value_of("max-flips") {
        None => 1000,
        Some(n) => n.parse().expect("Expected an integer"),
    };

    use rsat::Solution::*;
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
            println!("SAT");
            let solution = solution.iter().map(|&x| if x { 1 } else { -1 });
            for (i, v) in solution.enumerate() {
                print!("{} ", v * ((i + 1) as i32));
            }
            println!("0");
        }
    }

    let mut solver = rsat::msat::Solver::new();

    for _ in 0..formula.n_vars() {
        solver.new_var();
    }

    for rsat::Clause(lits) in formula.clauses {
        let mut cl = vec![];
        for rsat::Lit(l) in lits {
            let r = if l > 0 {
                (2 * (l - 1)) as usize
            } else {
                (2 * (-l - 1) + 1) as usize
            };
            cl.push(rsat::msat::Lit(r));
        }
        if !solver.new_clause(cl) {
            println!("UNSAT");
            return;
        }
    }

    if solver.solve() {
        println!("SAT");
        for v in 0..solver.n_vars() {
            match solver.value(v) {
                rsat::msat::LBool::False => print!("-{} ", v + 1),
                _ => print!("{} ", v + 1),
            }
        }
        println!("0");
    } else {
        println!("UNSAT");
    }
}
