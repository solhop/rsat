/// Branching heuristic to be used for cdcl
#[derive(Clone, Copy, Debug)]
pub enum BranchingHeuristic {
    /// VSIDS
    Vsids {
        /// Var increment
        var_inc: f64,
        ///Var decay
        var_decay: f64,
    },
    /// LRB
    Lrb,
}

/// Clause Db Options
#[derive(Clone, Copy, Debug)]
pub struct ClauseDbOptions {
    /// Clause increment
    pub cla_inc: f64,
    /// Clause decay
    pub cla_decay: f64,
}

/// Solver options.
pub struct SolverOptions {
    /// Clause Db Options
    pub clause_db_options: ClauseDbOptions,
    /// Branching Heuristic
    pub branching_heuristic: BranchingHeuristic,
    /// Should capture drat clauses
    pub capture_drat: bool,
}

impl Default for SolverOptions {
    fn default() -> Self {
        SolverOptions {
            clause_db_options: ClauseDbOptions {
                cla_inc: 1.0,
                cla_decay: 0.999,
            },
            branching_heuristic: BranchingHeuristic::Lrb,
            capture_drat: false,
        }
    }
}
