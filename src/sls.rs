use crate::common::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::BufRead;

/// Magic numbers used by local search.
const C_MAKE: f32 = 0.5;
const C_BREAK: f32 = 3.6;

/// A SAT Formula.
pub struct Formula {
    num_vars: usize,
    clauses: Vec<Clause>,
}

impl Formula {
    /// Read formula in DIMACS format from STDIN.
    pub fn new_from_stdin() -> Self {
        Formula::new_from_buf_reader(&mut std::io::stdin().lock())
    }

    /// Read formula in DIMACS format from a file.
    pub fn new_from_file(filename: &str) -> Self {
        let file = File::open(filename).expect("File not found");
        let mut reader = io::BufReader::new(file);
        Formula::new_from_buf_reader(&mut reader)
    }

    /// Read formula in DIMACS format from buffer reader.
    pub fn new_from_buf_reader<F>(reader: &mut F) -> Self
    where
        F: std::io::BufRead,
    {
        let mut n_clauses = 0usize;
        let mut f = Formula {
            num_vars: 0,
            clauses: vec![],
        };

        for line in reader.lines() {
            let line = line.unwrap();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
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
                    f.num_vars = n_vars;
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
                    let sign = if l < 0 { 1 } else { 0 };
                    let var = (l.abs() - 1) as usize;
                    let l = 2 * var + sign;
                    cl.push(Lit(l));
                }
                f.add_clause(cl);
                if f.clauses.len() == n_clauses {
                    break;
                }
            }
        }

        f
    }

    /// Returns the number of variables in the formula.
    pub fn n_vars(&self) -> usize {
        self.num_vars
    }

    /// Returns the number of clauses in the formula.
    pub fn n_clauses(&self) -> usize {
        self.clauses.len()
    }

    /// Add a clause to the formula.
    pub fn add_clause(&mut self, cl: Vec<Lit>) {
        self.clauses.push(Clause(cl));
    }

    /// Local Search based on probSAT. Tries for `max_tries` times
    /// with `max_flips` flips in each try.
    pub fn local_search(&mut self, max_tries: u32, max_flips: u32) -> Solution {
        let mut curr_model = vec![false; self.num_vars as usize];
        let mut best_model = vec![false; self.num_vars as usize];
        let mut best_n_unsat_clauses = self.clauses.len();

        let mut clause_unsat = vec![1; self.clauses.len()];

        let mut rng = thread_rng();

        for _ in 0..max_tries {
            Formula::gen_rand_model(&mut curr_model, &mut rng, &vec![LBool::None; self.num_vars]);
            for _ in 0..max_flips {
                let mut n_unsat_clauses = 0;
                for (i, Clause(cl)) in self.clauses.iter().enumerate() {
                    clause_unsat[i] = 1;
                    for &lit in cl {
                        let var = lit.var();
                        if lit.sign() != curr_model[var] {
                            clause_unsat[i] = 0;
                            break;
                        }
                    }
                    n_unsat_clauses += clause_unsat[i];
                }

                if n_unsat_clauses == 0 {
                    return Solution::Sat(curr_model.iter().copied().collect());
                } else if n_unsat_clauses < best_n_unsat_clauses {
                    best_model.clone_from_slice(&curr_model);
                    best_n_unsat_clauses = n_unsat_clauses;
                }

                let dist = WeightedIndex::new(&clause_unsat).unwrap();
                let selected_clause = dist.sample(&mut rng);

                let Clause(cl) = &self.clauses[selected_clause];
                let mut scores = vec![0.0; self.num_vars as usize];
                for x in cl {
                    let var_i = x.var();
                    let mut make_count = 0;
                    let mut break_count = 0;

                    curr_model[var_i] = !curr_model[var_i];
                    for (i, Clause(cl)) in self.clauses.iter().enumerate() {
                        let mut cl_unsat = 1;
                        for &lit in cl {
                            let var = lit.var();
                            if lit.sign() != curr_model[var] {
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
                    curr_model[var_i] = !curr_model[var_i];

                    scores[var_i] = C_MAKE.powi(make_count) / C_BREAK.powi(break_count);
                }

                let dist_var = WeightedIndex::new(&scores).unwrap();
                let selected_var = dist_var.sample(&mut rng);
                curr_model[selected_var] = !curr_model[selected_var];
            }
        }

        Solution::Best(best_model.iter().copied().collect())
    }

    fn gen_rand_model<T>(model: &mut Vec<bool>, rng: &mut T, l_model: &[LBool])
    where
        T: rand::Rng,
    {
        for (i, v) in model.iter_mut().enumerate() {
            match l_model[i] {
                LBool::None => *v = 2 * rng.gen_range(0, 2) - 1 == 1,
                LBool::True => *v = true,
                LBool::False => *v = false,
            }
        }
    }
}
