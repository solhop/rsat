use rsat::parse;
use std::env;
use std::error::Error;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No file name provided");
        return;
    }
    let formula = match parse(&args[1]) {
        Ok(f) => f,
        Err(e) => panic!("Error: {}", e.description()),
    };
    let solution = formula.solve();
    match solution {
        None => println!("UNSAT"),
        Some(solution) => {
            println!("SAT");
            for (i, v) in solution.iter().enumerate() {
                print!("{} ", v * ((i + 1) as i32));
            }
            println!("0");
        }
    }
}
