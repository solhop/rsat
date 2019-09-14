use std::env;
use std::error::Error;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No file name provided");
        return;
    }
    let formula = match rsat::Formula::new_from_file(&args[1]) {
        Ok(f) => f,
        Err(e) => panic!("Error: {}", e.description()),
    };

    use rsat::Solution::*;
    match formula.local_search() {
        Unsat => println!("UNSAT"),
        Best(solution) => {
            println!("UNKNOWN");
            for (i, v) in solution.iter().enumerate() {
                print!("{} ", v * ((i + 1) as i32));
            }
            println!("0");
        }
        Sat(solution) => {
            println!("SAT");
            for (i, v) in solution.iter().enumerate() {
                print!("{} ", v * ((i + 1) as i32));
            }
            println!("0");
        }
    }
}
