use crate::errors::*;
use crate::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rayon::prelude::*;
use std::fs::File;
use std::io;

/// Magic numbers used by local search.
const C_MAKE: f32 = 0.5;
const C_BREAK: f32 = 3.7;

/// Scoring function type.
pub enum ScoreFnType {
    /// Choose flip variable randomly.
    Rand,
    /// Use polynomial scoring function.
    Poly,
    /// Use exponential scoring function.
    Exp,
    /// Cutom scoring function.
    Custom(Box<dyn Fn(i32, i32) -> f32>),
}

/// SLS Solver.
pub struct Solver {
    num_vars: usize,
    clauses: Vec<Clause>,
}

impl Solver {
    /// Read formula in DIMACS format from STDIN.
    pub fn new_from_stdin() -> Result<Self> {
        Solver::new_from_buf_reader(&mut std::io::stdin().lock())
    }

    /// Read formula in DIMACS format from a file.
    pub fn new_from_file(filename: &str) -> Result<Self> {
        let file = File::open(filename).expect("File not found");
        let mut reader = io::BufReader::new(file);
        Solver::new_from_buf_reader(&mut reader)
    }

    /// Read formula in DIMACS format from buffer reader.
    pub fn new_from_buf_reader<F>(reader: &mut F) -> Result<Self>
    where
        F: std::io::BufRead,
    {
        let parsed = crate::parser::parse_dimacs_from_buf_reader(reader);
        match parsed {
            Ok(parsed) => {
                if let crate::parser::Dimacs::Cnf { n_vars, clauses } = parsed {
                    Ok(Solver {
                        num_vars: n_vars,
                        clauses: clauses
                            .into_iter()
                            .map(|cl| Clause {
                                lits: cl
                                    .into_iter()
                                    .map(|l| {
                                        let var = Var::new((l.abs() - 1) as usize);
                                        if l < 0 {
                                            var.neg()
                                        } else {
                                            var.pos()
                                        }
                                    })
                                    .collect(),
                            })
                            .collect(),
                    })
                } else {
                    panic!("Incorrect input format");
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Returns the number of variables in the formula.
    pub fn n_vars(&self) -> usize {
        self.num_vars
    }

    /// Returns the number of clauses in the formula.
    pub fn n_clauses(&self) -> usize {
        self.clauses.len()
    }

    /// Return's ith clause
    pub fn ith_clause(&self, i: usize) -> &Clause {
        &self.clauses[i]
    }

    /// Add a clause to the formula.
    pub fn add_clause(&mut self, lits: Vec<Lit>) {
        self.clauses.push(Clause { lits });
    }

    /// Local Search based on probSAT. Tries for `max_tries` times
    /// with `max_flips` flips in each try.
    pub fn local_search(
        &mut self,
        max_tries: u32,
        max_flips: u32,
        score_fn_type: ScoreFnType,
        parallel: bool,
    ) -> Solution {
        let mut curr_model = vec![false; self.num_vars as usize];
        let mut best_model = vec![false; self.num_vars as usize];
        let mut best_n_unsat_clauses = self.clauses.len();

        let mut clause_unsat = vec![1; self.clauses.len()];

        let mut rng = thread_rng();

        for _ in 0..max_tries {
            Solver::gen_rand_model(
                &mut curr_model,
                &mut rng,
                &vec![LBool::Undef; self.num_vars],
            );

            for _ in 0..max_flips {
                let n_unsat_clauses = if parallel {
                    self.clauses
                        .par_iter()
                        .zip(clause_unsat.par_iter_mut())
                        .map(|(cl, cl_us)| {
                            let mut clause_unsat = 1;
                            for lit in &cl.lits {
                                let var = lit.var();
                                if lit.sign() != curr_model[var.index()] {
                                    clause_unsat = 0;
                                    break;
                                }
                            }
                            *cl_us = clause_unsat;
                            clause_unsat
                        })
                        .sum()
                } else {
                    self.clauses
                        .iter()
                        .zip(clause_unsat.iter_mut())
                        .map(|(cl, cl_us)| {
                            let mut clause_unsat = 1;
                            for lit in &cl.lits {
                                let var = lit.var();
                                if lit.sign() != curr_model[var.index()] {
                                    clause_unsat = 0;
                                    break;
                                }
                            }
                            *cl_us = clause_unsat;
                            clause_unsat
                        })
                        .sum()
                };

                if n_unsat_clauses == 0 {
                    return Solution::Sat(curr_model.iter().copied().collect());
                } else if n_unsat_clauses < best_n_unsat_clauses {
                    best_model.clone_from_slice(&curr_model);
                    best_n_unsat_clauses = n_unsat_clauses;
                }

                let dist = WeightedIndex::new(&clause_unsat).unwrap();
                let selected_clause = dist.sample(&mut rng);

                let Clause { lits: cl } = &self.clauses[selected_clause];
                let mut scores = vec![0.0; self.num_vars as usize];
                for x in cl {
                    let var_i = x.var();

                    curr_model[var_i.index()] = !curr_model[var_i.index()];
                    let (break_count, make_count) = if parallel {
                        self.clauses
                            .par_iter()
                            .zip(clause_unsat.par_iter())
                            .map(|(Clause { lits: cl }, cl_us)| {
                                let mut cl_unsat = 1;
                                for &lit in cl {
                                    let var = lit.var();
                                    if lit.sign() != curr_model[var.index()] {
                                        cl_unsat = 0;
                                        break;
                                    }
                                }

                                if cl_unsat != *cl_us {
                                    if cl_unsat == 1 {
                                        // break_count += 1;
                                        (1, 0)
                                    } else {
                                        // make_count += 1;
                                        (0, 1)
                                    }
                                } else {
                                    (0, 0)
                                }
                            })
                            .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
                    } else {
                        self.clauses
                            .iter()
                            .zip(clause_unsat.iter())
                            .map(|(Clause { lits: cl }, cl_us)| {
                                let mut cl_unsat = 1;
                                for &lit in cl {
                                    let var = lit.var();
                                    if lit.sign() != curr_model[var.index()] {
                                        cl_unsat = 0;
                                        break;
                                    }
                                }

                                if cl_unsat != *cl_us {
                                    if cl_unsat == 1 {
                                        // break_count += 1;
                                        (1, 0)
                                    } else {
                                        // make_count += 1;
                                        (0, 1)
                                    }
                                } else {
                                    (0, 0)
                                }
                            })
                            .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
                    };

                    curr_model[var_i.index()] = !curr_model[var_i.index()];

                    scores[var_i.index()] = match &score_fn_type {
                        ScoreFnType::Rand => 1.0,
                        ScoreFnType::Poly => 1.0 / (1.0 + break_count as f32).powf(C_BREAK),
                        ScoreFnType::Exp => C_MAKE.powi(make_count) / C_BREAK.powi(break_count),
                        ScoreFnType::Custom(f) => f(make_count, break_count),
                    };
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
                LBool::Undef => *v = 2 * rng.gen_range(0, 2) - 1 == 1,
                LBool::True => *v = true,
                LBool::False => *v = false,
            }
        }
    }

    /// Verify that the clauses are satisfied by the input model.
    pub fn verify(&self, model: &[bool]) -> bool {
        // println!("c Verifying solution");
        for Clause { lits: cl } in self.clauses.iter() {
            let mut cla_sat = false;
            for &lit in cl.iter() {
                let var = lit.var();
                if lit.sign() != model[var.index()] {
                    cla_sat = true;
                    break;
                }
            }
            if !cla_sat {
                return false;
            }
        }
        true
    }
}
