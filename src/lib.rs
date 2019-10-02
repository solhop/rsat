//! `rsat` is a SAT and MaxSAT Solver.
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
//!     println!("{:?}", rsat::sls::Formula::new_from_buf_reader(&mut input.as_bytes())
//!         .local_search(10, 100, rsat::sls::ScoreFnType::Exp, false));
//! }
//! ```

#![deny(missing_docs)]

/// Common utils.
pub mod common;

/// sls, a local search solver module.
pub mod sls;

/// msat, a complete CDCL solver module.
pub mod msat;
