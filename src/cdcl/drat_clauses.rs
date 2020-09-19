use crate::Lit;

/// Storage for drat clauses
pub(crate) struct DratClauses {
    pub drat_clauses: Vec<(Vec<Lit>, bool)>,
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
            self.drat_clauses.push((Vec::from(lits), is_delete));
        }
    }
}
