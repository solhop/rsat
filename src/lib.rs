//! `rsat` is a SAT Solver.
//!
//! ## An example using the SLS solver
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
//!     println!("{:?}", rsat::sls::Solver::new_from_buf_reader(&mut input.as_bytes())
//!         .unwrap().local_search(10, 100, rsat::sls::ScoreFnType::Exp, false));
//! }
//! ```

#![deny(missing_docs)]

/// Common utils.
mod common;

pub use common::*;

/// DIMACS Parser.
pub mod parser;

/// sls, a local search solver module.
pub mod sls;

/// msat, a complete CDCL solver module.
pub mod msat;
