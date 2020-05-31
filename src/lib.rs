//! `rsat` is a SAT Solver.
//!
//! ## An example using the SLS solver
//!
//! ```rust
//! use rsat::cdcl::{Solver, SolverOptions};
//! use rsat::{Var, Solution};
//!
//! let options = SolverOptions::default();
//! let mut solver = Solver::new(options);
//! let vars: Vec<Var> = solver.new_vars(3);
//! solver.add_clause(vec![vars[0].pos()]);
//! solver.add_clause(vec![vars[1].neg()]);
//! solver.add_clause(vec![vars[0].neg(), vars[1].pos(), vars[2].pos()]);
//!
//! assert_eq!(solver.solve(vec![]), Solution::Sat(vec![true, false, true]));
//!
//! assert_eq!(solver.solve(vec![vars[2].neg()]), Solution::Unsat);
//!
//! assert_eq!(solver.solve(vec![]), Solution::Sat(vec![true, false, true]));
//!
//! solver.add_clause(vec![vars[2].neg()]);
//! assert_eq!(solver.solve(vec![]), Solution::Unsat);
//! ```

#![deny(missing_docs)]

/// Common utils.
mod common;

pub use common::*;

/// DIMACS Parser.
pub mod parser;

/// sls, a local search solver module.
pub mod sls;

/// CDCL solver module.
pub mod cdcl;
