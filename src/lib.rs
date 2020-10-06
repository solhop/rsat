//! `rsat` is a SAT Solver.
//!
//! ## An example using the CDCL solver
//!
//! ```rust
//! use rsat::cdcl::{Solver, SolverOptions};
//! use rsat::common::{Var, Solution};
//!
//! let options = SolverOptions::default();
//! let mut solver = Solver::new(options);
//! let vars: Vec<Var> = solver.new_vars(3);
//! solver.add_clause(vec![vars[0].pos_lit()]);
//! solver.add_clause(vec![vars[1].neg_lit()]);
//! solver.add_clause(vec![vars[0].neg_lit(), vars[1].pos_lit(), vars[2].pos_lit()]);
//!
//! assert_eq!(solver.solve(vec![]), Solution::Sat(vec![true, false, true]));
//!
//! assert_eq!(solver.solve(vec![vars[2].neg_lit()]), Solution::Unsat);
//!
//! assert_eq!(solver.solve(vec![]), Solution::Sat(vec![true, false, true]));
//!
//! solver.add_clause(vec![vars[2].neg_lit()]);
//! assert_eq!(solver.solve(vec![]), Solution::Unsat);
//! ```

#![deny(missing_docs)]

/// Common utils.
pub mod common;

/// DIMACS Parser.
pub mod parser;

/// Stochastic local search solver module.
pub mod sls;

/// CDCL solver module.
pub mod cdcl;
