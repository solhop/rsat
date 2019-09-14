use rand::distributions::WeightedIndex;
use rand::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::BufRead;

/// Magic numbers
const C_MAKE: f32 = 0.5;
const C_BREAK: f32 = 3.6;

pub struct Lit(i32);

pub struct Clause(Vec<Lit>);

pub struct Formula {
    num_vars: u32,
    clauses: Vec<Clause>,
}

pub enum Solution {
    Unsat,
    Best(Vec<i32>),
    Sat(Vec<i32>),
}

impl Formula {
    pub fn new_from_file(filename: &str) -> io::Result<Formula> {
        let file = File::open(filename)?;
        let reader = io::BufReader::new(file);
        let mut n_clauses = 0usize;
        let mut f = Formula {
            num_vars: 0,
            clauses: vec![],
        };

        for line in reader.lines() {
            let line = line?;
            if line.starts_with('c') {
                continue;
            } else if line.starts_with('p') {
                let re = Regex::new(r"p\s+cnf\s+(\d+)\s+(\d+)").unwrap();
                let cap = re.captures(&line);
                if let Some(cap) = cap {
                    let n_vars = match cap[1].parse() {
                        Ok(n) => n,
                        _ => panic!("Input file could not be parsed"),
                    };
                    n_clauses = match cap[2].parse() {
                        Ok(n) => n,
                        _ => panic!("Input file could not be parsed"),
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
                        _ => panic!("Invalid character"),
                    };
                    cl.push(Lit(l));
                }
                f.add_clause(cl);
                if f.clauses.len() == n_clauses {
                    break;
                }
            }
        }

        Ok(f)
    }

    fn set_num_vars(&mut self, n_vars: u32) {
        self.num_vars = n_vars;
    }

    pub fn n_vars(&self) -> u32 {
        self.num_vars
    }

    pub fn add_clause(&mut self, cl: Vec<Lit>) {
        self.clauses.push(Clause(cl));
    }

    pub fn local_search(&self) -> Solution {
        let mut curr_model = vec![-1; self.num_vars as usize];
        let mut best_model = vec![-1; self.num_vars as usize];
        let mut best_n_unsat_clauses = self.clauses.len();

        let max_tries = 10;
        let max_flips = 100;

        let mut clause_unsat = vec![1; self.clauses.len()];

        let mut rng = thread_rng();

        for _ in 0..max_tries {
            Formula::gen_rand_model(&mut curr_model, &mut rng);
            for _ in 0..max_flips {
                let mut n_unsat_clauses = 0;
                for (i, Clause(cl)) in self.clauses.iter().enumerate() {
                    clause_unsat[i] = 1;
                    for Lit(lit) in cl {
                        let var = lit.abs();
                        if *lit == var * curr_model[(var - 1) as usize] {
                            clause_unsat[i] = 0;
                            break;
                        }
                    }
                    n_unsat_clauses += clause_unsat[i];
                }

                if n_unsat_clauses == 0 {
                    return Solution::Sat(curr_model);
                } else if n_unsat_clauses < best_n_unsat_clauses {
                    best_model.clone_from_slice(&curr_model);
                    best_n_unsat_clauses = n_unsat_clauses;
                }

                let dist = WeightedIndex::new(&clause_unsat).unwrap();
                let selected_clause = dist.sample(&mut rng);

                let Clause(cl) = &self.clauses[selected_clause];
                let mut scores = vec![0.0; self.num_vars as usize];
                for Lit(x) in cl {
                    let var_i = (x.abs() - 1) as usize;
                    let mut make_count = 0;
                    let mut break_count = 0;

                    curr_model[var_i] = -curr_model[var_i];
                    for (i, Clause(cl)) in self.clauses.iter().enumerate() {
                        let mut cl_unsat = 1;
                        for Lit(lit) in cl {
                            let var = lit.abs();
                            if *lit == var * curr_model[(var - 1) as usize] {
                                cl_unsat = 0;
                                break;
                            }
                        }

                        if cl_unsat != clause_unsat[i] {
                            if cl_unsat == 1 {
                                break_count += 1;
                            } else {
                                make_count += 1;
                            }
                        }
                    }
                    curr_model[var_i] = -curr_model[var_i];

                    scores[var_i] = C_MAKE.powi(make_count) / C_BREAK.powi(break_count);
                }

                let dist_var = WeightedIndex::new(&scores).unwrap();
                let selected_var = dist_var.sample(&mut rng);
                curr_model[selected_var] = -curr_model[selected_var];
            }
        }

        Solution::Best(best_model)
    }

    fn gen_rand_model<T>(model: &mut Vec<i32>, rng: &mut T)
    where
        T: rand::Rng,
    {
        for v in model.iter_mut() {
            *v = 2 * rng.gen_range(0, 2) - 1;
        }
    }

    #[allow(dead_code)]
    fn solve(&self) -> Option<Vec<i32>> {
        let mut sat = vec![];
        let mut model = vec![];
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
