use super::solver_options::ClauseDbOptions;
use super::{DratClauses, VarManager};
use crate::*;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub enum ClauseRef {
    Orig(usize),
    Lrnt(Weak<RefCell<(Clause, f64)>>),
}

impl PartialEq for ClauseRef {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ClauseRef::Orig(i), ClauseRef::Orig(j)) => i == j,
            (ClauseRef::Lrnt(lhs_ref), ClauseRef::Lrnt(rhs_ref)) => {
                lhs_ref.as_ptr() == rhs_ref.as_ptr()
            }
            _ => false,
        }
    }
}

// #[derive(Clone, Copy, PartialEq, Debug)]
// pub enum ClauseIndex {
//     Orig(usize),
//     Lrnt(usize),
// }

pub struct ClauseDb {
    original: Vec<Clause>,
    learnt_refs: Vec<Rc<RefCell<(Clause, f64)>>>,
    cla_inc: f64,
    cla_decay: f64,
}

impl ClauseDb {
    pub fn new(options: ClauseDbOptions) -> Self {
        ClauseDb {
            original: vec![],
            learnt_refs: Vec::new(),
            cla_inc: options.cla_inc,
            cla_decay: 1.0 / options.cla_decay,
        }
    }

    pub fn original_len(&self) -> usize {
        self.original.len()
    }

    pub fn learnts_len(&self) -> usize {
        self.learnt_refs.len()
    }

    pub fn add_original(&mut self, cl: Clause) -> ClauseRef {
        let ci = ClauseRef::Orig(self.original.len());
        self.original.push(cl);
        ci
    }

    pub fn add_learnt(&mut self, cl: Clause) -> ClauseRef {
        let learnt_clause = Rc::new(RefCell::new((cl, 0.0)));
        let clause_ref = ClauseRef::Lrnt(Rc::downgrade(&learnt_clause));
        self.learnt_refs.push(learnt_clause);
        self.found_clause_as_reason(clause_ref.clone());
        clause_ref
    }

    pub fn get_original_mut(&mut self, index: usize) -> Option<&mut Clause> {
        self.original.get_mut(index)
    }

    pub fn get_clause_ref(&self, ci: ClauseRef) -> Option<&Clause> {
        match ci {
            ClauseRef::Orig(ci) => Some(&self.original[ci]),
            ClauseRef::Lrnt(ci) => ci.upgrade().map(|cl| &cl.borrow().0),
        }
    }

    pub fn get_clause_mut_ref(&mut self, ci: ClauseRef) -> Option<&mut Clause> {
        match ci {
            ClauseRef::Orig(ci) => Some(&mut self.original[ci]),
            ClauseRef::Lrnt(ci) => ci.upgrade().map(|cl| &mut cl.borrow_mut().0),
        }
    }

    pub fn found_clause_as_reason(&mut self, ci: ClauseRef) {
        if let ClauseRef::Lrnt(clause_ref) = ci {
            if let Some(cl_ref) = clause_ref.upgrade() {
                let cl_mut = cl_ref.borrow_mut();
                cl_mut.1 += self.cla_inc;
                if cl_mut.1 > 1e100 {
                    for cl in self.learnt_refs.iter_mut() {
                        cl.borrow_mut().1 *= 1e-100;
                    }
                    self.cla_inc *= 1e-100;
                }
            }
        }
    }

    pub fn after_record_learnt_clause(&mut self) {
        self.cla_inc *= self.cla_decay;
    }

    /// If the clause is reason for some variable
    /// (INVARIANT: if it is, then it should be var corresponding to first literal),
    /// then the clause is locked.
    fn is_clause_locked(&self, ci: ClauseRef, var_manager: &VarManager) -> bool {
        let cl = self.get_clause_ref(ci);
        match cl {
            Some(cl) => true, // TODO FIXME var_manager.get_reason(cl.lits[0].var()) == Some(ci),
            None => false,
        }
    }

    pub(crate) fn reduce_db(
        &mut self,
        var_manager: &VarManager,
        watches: &mut Vec<Vec<ClauseRef>>,
        drat_clauses: &mut DratClauses,
    ) {
        let lim = self.cla_inc / self.learnt_refs.len() as f64;

        let mut acts: Vec<(Weak<RefCell<(Clause, f64)>>, f64, usize)> = self
            .learnt_refs
            .iter()
            .map(|cl_rc| {
                let cl_ref = cl_rc.borrow();
                let cl = cl_ref.0;
                let a = cl_ref.1;
                (Rc::downgrade(cl_rc), a, cl.lits.len())
            })
            .collect();
        // Using clause length does help (TODO)
        // acts.sort_by(|(_, a1, l1), (_, a2, l2)| match l2.cmp(l1) {
        //     std::cmp::Ordering::Less => std::cmp::Ordering::Less,
        //     std::cmp::Ordering::Equal => a1.partial_cmp(a2).unwrap(),
        //     std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        // });
        acts.sort_by(|(_, a1, _), (_, a2, _)| a1.partial_cmp(a2).unwrap());

        let mut i = 0;
        while i < acts.len() / 2 {
            let cl_ref = acts[i].0;
            if !self.is_clause_locked(ClauseRef::Lrnt(cl_ref), var_manager) {
                self.remove_learnt(cl_ref, watches, drat_clauses);
            }
            i += 1;
        }

        while i < self.learnt_refs.len() {
            let cl_ref = acts[i].0;
            if !self.is_clause_locked(ClauseRef::Lrnt(cl_ref), var_manager) && acts[i].1 < lim {
                self.remove_learnt(cl_ref, watches, drat_clauses);
            }
            i += 1;
        }
    }

    pub(crate) fn remove_learnt(
        &mut self,
        cl_weak_ref: Weak<RefCell<(Clause, f64)>>,
        watches: &mut Vec<Vec<ClauseRef>>,
        drat_clauses: &mut DratClauses,
    ) {
        if let Some(cl_ref) = cl_weak_ref.upgrade() {
            let learnt_with_index = self
                .learnt_refs
                .iter()
                .enumerate()
                .find(|(index, cl)| cl.as_ptr() == cl_ref.as_ptr());
            if let Some(learnt_with_index) = learnt_with_index {
                let index = learnt_with_index.0;
                let learnt = learnt_with_index.1.borrow().0;
                if let Some(i) = watches[(!learnt.lits[0]).index()]
                    .iter()
                    .position(|&s| s == ClauseRef::Lrnt(cl_weak_ref))
                {
                    watches[(!learnt.lits[0]).index()].remove(i);
                }
                if let Some(i) = watches[(!learnt.lits[1]).index()]
                    .iter()
                    .position(|&s| s == ClauseRef::Lrnt(cl_weak_ref))
                {
                    watches[(!learnt.lits[1]).index()].remove(i);
                }

                drat_clauses.capture(&learnt.lits, true);
                self.learnt_refs.remove(index);
            }
        }
    }

    pub fn learnt_indices(&self) -> Vec<Weak<RefCell<(Clause, f64)>>> {
        self.learnt_refs
            .iter()
            .map(|rc| Rc::downgrade(rc))
            .collect()
    }
}
