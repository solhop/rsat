use super::{DratClauses, VarManager};
use crate::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ClauseIndex {
    Orig(usize),
    Lrnt(usize),
}

pub struct ClauseDb {
    original: Vec<Clause>,
    learnts: HashMap<usize, (Clause, f64)>,
    curr_learnt_id: usize,
    cla_inc: f64,
    cla_decay: f64,
}

impl ClauseDb {
    pub fn new(cla_inc: f64, cla_decay: f64) -> Self {
        ClauseDb {
            original: vec![],
            learnts: HashMap::new(),
            curr_learnt_id: 0,
            cla_inc,
            cla_decay: 1.0 / cla_decay,
        }
    }

    pub fn original_len(&self) -> usize {
        self.original.len()
    }

    pub fn learnts_len(&self) -> usize {
        self.learnts.len()
    }

    pub fn add_original(&mut self, cl: Clause) -> ClauseIndex {
        let ci = ClauseIndex::Orig(self.original.len());
        self.original.push(cl);
        ci
    }

    pub fn add_learnt(&mut self, cl: Clause) -> ClauseIndex {
        self.learnts.insert(self.curr_learnt_id, (cl, 0.0));
        let ci = ClauseIndex::Lrnt(self.curr_learnt_id);
        self.curr_learnt_id += 1;
        self.found_clause_as_reason(ci);
        ci
    }

    pub fn get_original_mut(&mut self, index: usize) -> Option<&mut Clause> {
        self.original.get_mut(index)
    }

    pub fn get_learnt_mut(&mut self, index: usize) -> Option<&mut Clause> {
        self.learnts.get_mut(&index).map(|(c, _)| c)
    }

    pub fn get_clause_ref(&self, ci: ClauseIndex) -> &Clause {
        match ci {
            ClauseIndex::Orig(ci) => &self.original[ci],
            ClauseIndex::Lrnt(ci) => &self.learnts.get(&ci).map(|(c, _)| c).unwrap(),
        }
    }

    pub fn get_clause_mut_ref(&mut self, ci: ClauseIndex) -> &mut Clause {
        match ci {
            ClauseIndex::Orig(ci) => &mut self.original[ci],
            ClauseIndex::Lrnt(ci) => self.learnts.get_mut(&ci).map(|(c, _)| c).unwrap(),
        }
    }

    pub fn found_clause_as_reason(&mut self, ci: ClauseIndex) {
        if let ClauseIndex::Lrnt(index) = ci {
            let cl = self.learnts.get_mut(&index).unwrap();
            cl.1 += self.cla_inc;
            if cl.1 > 1e100 {
                for (_, cl) in self.learnts.iter_mut() {
                    cl.1 *= 1e-100;
                }
                self.cla_inc *= 1e-100;
            }
        }
    }

    pub fn after_record_learnt_clause(&mut self) {
        self.cla_inc *= self.cla_decay;
    }

    /// If the clause is reason for some variable
    /// (INVARIANT: if it is, then it should be var corresponding to first literal),
    /// then the clause is locked.
    fn is_clause_locked(&self, ci: ClauseIndex, var_manager: &VarManager) -> bool {
        let cl = self.get_clause_ref(ci);
        var_manager.get_reason(cl.lits[0].var()) == Some(ci)
    }

    pub(crate) fn reduce_db(
        &mut self,
        var_manager: &VarManager,
        watches: &mut Vec<Vec<ClauseIndex>>,
        drat_clauses: &mut DratClauses,
    ) {
        let mut i = 0;
        let lim = self.cla_inc / self.learnts.len() as f64;

        let mut acts: Vec<(usize, f64, usize)> = self
            .learnts
            .iter()
            .map(|(&i, (cl, a))| (i, *a, cl.lits.len()))
            .collect();
        // Using clause length does help (TODO)
        // acts.sort_by(|(_, a1, l1), (_, a2, l2)| match l2.cmp(l1) {
        //     std::cmp::Ordering::Less => std::cmp::Ordering::Less,
        //     std::cmp::Ordering::Equal => a1.partial_cmp(a2).unwrap(),
        //     std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        // });
        acts.sort_by(|(_, a1, _), (_, a2, _)| a1.partial_cmp(a2).unwrap());

        while i < acts.len() / 2 {
            let index = acts[i].0;
            let ci = ClauseIndex::Lrnt(index);
            if !self.is_clause_locked(ci, var_manager) {
                self.remove_learnt(index, watches, drat_clauses);
            }
            i += 1;
        }

        while i < self.learnts.len() {
            let index = acts[i].0;
            let ci = ClauseIndex::Lrnt(index);
            if !self.is_clause_locked(ci, var_manager) && acts[i].1 < lim {
                self.remove_learnt(index, watches, drat_clauses);
            }
            i += 1;
        }
    }

    pub(crate) fn remove_learnt(
        &mut self,
        index: usize,
        watches: &mut Vec<Vec<ClauseIndex>>,
        drat_clauses: &mut DratClauses,
    ) {
        let learnt = self.learnts.get(&index).map(|(c, _)| c).unwrap();
        if let Some(i) = watches[(!learnt.lits[0]).index()]
            .iter()
            .position(|&s| s == ClauseIndex::Lrnt(index))
        {
            watches[(!learnt.lits[0]).index()].remove(i);
        }
        if let Some(i) = watches[(!learnt.lits[1]).index()]
            .iter()
            .position(|&s| s == ClauseIndex::Lrnt(index))
        {
            watches[(!learnt.lits[1]).index()].remove(i);
        }
        drat_clauses.capture(&learnt.lits, true);
        self.learnts.remove(&index);
    }

    pub fn learnt_indices(&self) -> Vec<usize> {
        self.learnts.iter().map(|(&i, _)| i).collect()
    }
}
