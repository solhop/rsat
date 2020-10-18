use solhop_types::Lit;

/// Drat Clause type
pub enum DratClause {
    /// Represents Add Drat Clause
    Add(Vec<Lit>),
    /// Represents Delete Drat Clause
    Delete(Vec<Lit>),
}

/// Storage for drat clauses
pub(crate) struct DratClauses {
    drat_clauses: Vec<DratClause>,
    capture_drat: bool,
}

impl DratClauses {
    pub fn new(capture_drat: bool) -> Self {
        Self {
            drat_clauses: vec![],
            capture_drat,
        }
    }

    pub fn capture(&mut self, lits: &[Lit], is_delete: bool) {
        if self.capture_drat {
            self.drat_clauses.push(if is_delete {
                DratClause::Delete(Vec::from(lits))
            } else {
                DratClause::Add(Vec::from(lits))
            });
        }
    }

    pub fn drat_clauses(self) -> Option<Vec<DratClause>> {
        if self.capture_drat {
            Some(self.drat_clauses)
        } else {
            None
        }
    }
}
