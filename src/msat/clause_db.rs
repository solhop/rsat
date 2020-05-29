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
            cla_decay,
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
        ci
    }

    pub fn get_cla_inc(&self) -> f64 {
        self.cla_inc
    }

    // pub fn get_original(&self, index: usize) -> Option<&Clause> {
    //     self.original.get(index)
    // }

    pub fn get_original_mut(&mut self, index: usize) -> Option<&mut Clause> {
        self.original.get_mut(index)
    }

    pub fn get_learnt(&self, index: usize) -> Option<&Clause> {
        self.learnts.get(&index).map(|(c, _)| c)
    }

    pub fn get_learnt_mut(&mut self, index: usize) -> Option<&mut Clause> {
        self.learnts.get_mut(&index).map(|(c, _)| c)
    }

    pub fn remove_learnt(&mut self, index: usize) {
        self.learnts.remove(&index);
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

    pub fn cla_bump_activity(&mut self, ci: ClauseIndex) {
        if let ClauseIndex::Lrnt(index) = ci {
            let cl = self.learnts.get_mut(&index).unwrap();
            cl.1 += self.cla_inc;
            if cl.1 > 1e100 {
                self.cla_rescale_activity();
            }
        }
    }

    pub fn cla_decay_activity(&mut self) {
        self.cla_inc *= self.cla_decay;
    }

    pub fn cla_rescale_activity(&mut self) {
        for (_, cl) in self.learnts.iter_mut() {
            cl.1 *= 1e-100;
        }
        self.cla_inc *= 1e-100;
    }

    pub fn update_cla_decay(&mut self, cla_decay: f64) {
        self.cla_decay = cla_decay;
    }

    pub fn learnt_activities(&self) -> Vec<(usize, f64, usize)> {
        self.learnts
            .iter()
            .map(|(&i, (cl, a))| (i, *a, cl.lits.len()))
            .collect()
    }

    pub fn learnt_indices(&self) -> Vec<usize> {
        self.learnts.iter().map(|(&i, _)| i).collect()
    }
}
