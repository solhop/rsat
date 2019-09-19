//! `rsat` is a SAT and MaxSAT Solver.
//!
//! Currently, it implements Local Search based on probSAT.
//!
//! ## An example
//!
//! ```rust
//! fn main() {
//!     let input = "
//!     c SAT instance
//!     p cnf 3 4
//!     1 0
//!     -1 -2 0
//!     2 -3 0
//!     -3 0
//!     ";
//!     println!("{:?}", rsat::Formula::new_from_buf_reader(&mut input.as_bytes()).local_search(10, 100));
//! }
//! ```

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::BufRead;

/// Magic numbers used by local search
const C_MAKE: f32 = 0.5;
const C_BREAK: f32 = 3.6;

/// A literal.
#[derive(Debug, Clone)]
pub struct Lit(pub i32);

/// A Clause.
#[derive(Debug)]
pub struct Clause(pub Vec<Lit>);

/// A SAT Formula
#[derive(Debug)]
pub struct Formula {
    num_vars: u32,
    pub clauses: Vec<Clause>, // TODO Temp pub
}

/// Lifted Boolean
#[derive(Debug, Clone, PartialEq)]
pub enum LBool {
    True,
    False,
    None,
}

/// Solution to the SAT Formula.
#[derive(Debug)]
pub enum Solution {
    /// The formula is unsatisfiable
    Unsat,
    /// Neither SAT or UNSAT was proven. Best model known so far.
    Best(Vec<bool>),
    /// The formula is satisfiable. A satifying model for the formula.
    Sat(Vec<bool>),
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
    pub fn n_vars(&self) -> u32 {
        self.num_vars
    }

    /// Returns the number of clauses in the formula.
    pub fn n_clauses(&self) -> u32 {
        self.clauses.len() as u32
    }

    /// Add a clause to the formula.
    pub fn add_clause(&mut self, cl: Vec<Lit>) {
        self.clauses.push(Clause(cl));
    }

    /// Local Search based on probSAT. Tries for `max_tries` times
    /// with `max_flips` flips in each try.
    pub fn local_search(&mut self, max_tries: u32, max_flips: u32) -> Solution {
        let l_model = self.simplify();
        let mut curr_model = vec![-1; self.num_vars as usize];
        let mut best_model = vec![-1; self.num_vars as usize];
        let mut best_n_unsat_clauses = self.clauses.len();

        let mut clause_unsat = vec![1; self.clauses.len()];

        let mut rng = thread_rng();

        for _ in 0..max_tries {
            Formula::gen_rand_model(&mut curr_model, &mut rng, &l_model);
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
                    return Solution::Sat(curr_model.iter().map(|&x| x == 1).collect());
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

        Solution::Best(best_model.iter().map(|&x| x == 1).collect())
    }

    /// Simplify the formula by performing unit propagation
    /// Returns false if formula is found unsat
    pub fn simplify(&mut self) -> Vec<LBool> {
        let mut model = vec![LBool::None; self.num_vars as usize];
        let mut cl_sat = vec![false; self.clauses.len()];
        let mut simplified = true;
        while simplified {
            simplified = false;
            for (i, Clause(cl)) in self.clauses.iter().enumerate() {
                if cl_sat[i] {
                    continue;
                }
                let mut n_unassigned = 0;
                let mut unassigned_lit = 0;
                for &Lit(l) in cl {
                    let var = l.abs();
                    let var_i = (var - 1) as usize;
                    if model[var_i] == LBool::None {
                        n_unassigned += 1;
                        unassigned_lit = l;
                    }
                    if (l > 0 && model[var_i] == LBool::True)
                        || (l < 0 && model[var_i] == LBool::False)
                    {
                        cl_sat[i] = true;
                        break;
                    }
                }
                if cl_sat[i] {
                    continue;
                }
                if n_unassigned == 1 {
                    if unassigned_lit > 0 {
                        model[(unassigned_lit - 1) as usize] = LBool::True;
                    } else {
                        model[(-unassigned_lit - 1) as usize] = LBool::False;
                    }
                    cl_sat[i] = true;
                    simplified = true;
                }
            }
        }
        let mut clauses = vec![];
        for (i, Clause(cl)) in self.clauses.iter().enumerate() {
            if !cl_sat[i] {
                clauses.push(Clause(cl.to_vec()));
            }
        }
        self.clauses = clauses;
        model
    }

    fn gen_rand_model<T>(model: &mut Vec<i32>, rng: &mut T, l_model: &[LBool])
    where
        T: rand::Rng,
    {
        for (i, v) in model.iter_mut().enumerate() {
            match l_model[i] {
                LBool::None => *v = 2 * rng.gen_range(0, 2) - 1,
                LBool::True => *v = 1,
                LBool::False => *v = -1,
            }
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

/// msat, a complete solver based on MiniSAT
pub mod msat;
