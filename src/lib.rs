use regex::Regex;
use std::fs::File;
use std::io;
use std::io::BufRead;

pub struct Lit(i32);

pub struct Clause(Vec<Lit>);

pub struct Formula {
    num_vars: u32,
    clauses: Vec<Clause>,
}

impl Formula {
    pub fn new() -> Formula {
        Formula {
            num_vars: 0,
            clauses: vec![],
        }
    }

    pub fn set_num_vars(&mut self, n_vars: u32) {
        self.num_vars = n_vars;
    }

    pub fn n_vars(&self) -> u32 {
        self.num_vars
    }

    pub fn add_clause(&mut self, cl: Vec<Lit>) {
        self.clauses.push(Clause(cl));
    }

    pub fn solve(&self) -> Option<Vec<i32>> {
        let mut sat = Vec::new();
        let mut model = Vec::new();
        for _ in 0..self.n_vars() {
            model.push(-1);
        }
        for _ in self.clauses.iter() {
            sat.push(false);
        }
        loop {
            for (i, Clause(c)) in self.clauses.iter().enumerate() {
                let mut s = false;
                for Lit(l) in c {
                    let v = l.abs();
                    if model[(v - 1) as usize] * v == *l {
                        s = true;
                        break;
                    }
                }
                sat[i] = s;
            }
            if !sat.contains(&false) {
                return Some(model);
            }
            if !model.contains(&-1) {
                break;
            }
            for v in model.iter_mut() {
                if *v == -1 {
                    *v = 1;
                    break;
                } else {
                    *v = -1;
                }
            }
        }
        None
    }
}

pub fn parse(filename: &str) -> io::Result<Formula> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut n_clauses = 0usize;
    let mut f = Formula::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("c") {
            continue;
        } else if line.starts_with("p") {
            let re = Regex::new(r"p\s+cnf\s+(\d+)\s+(\d+)").unwrap();
            let cap = re.captures(&line);
            if let Some(cap) = cap {
                let n_vars = match cap[1].parse() {
                    Ok(n) => n,
                    Err(_) => panic!("Input file could not be parsed"),
                };
                n_clauses = match cap[2].parse() {
                    Ok(n) => n,
                    Err(_) => panic!("Input file could not be parsed"),
                };
                f.set_num_vars(n_vars);
            }
        } else {
            let re = Regex::new(r"(-?\d+)").unwrap();
            let mut cl = vec![];
            for cap in re.captures_iter(&line) {
                let l = match cap[1].parse::<i32>() {
                    Ok(0) => continue,
                    Ok(n) => n,
                    Err(_) => panic!("Invalid character"),
                };
                cl.push(Lit(l));
            }
            f.add_clause(cl);
        }
    }

    if n_clauses != f.clauses.len() {
        panic!("Incorrect number of clauses");
    }

    Ok(f)
}
