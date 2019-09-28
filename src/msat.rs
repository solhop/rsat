#![allow(dead_code)]

use std::collections::VecDeque;

use crate::common::{Clause, LBool, Lit, Var};

#[derive(Clone, Copy, PartialEq)]
enum ClauseIndex {
    Orig(usize),
    Lrnt(usize),
}

pub struct Solver {
    clauses: Vec<Clause>,
    learnts: Vec<Clause>,
    cla_inc: f64,
    cla_decay: f64,
    var_inc: f64,
    var_decay: f64,
    activity: Vec<f64>,
    cla_activity: Vec<f64>,
    watches: Vec<Vec<ClauseIndex>>,
    undos: Vec<Vec<ClauseIndex>>,
    prop_q: VecDeque<Lit>,
    assigns: Vec<LBool>,
    trail: Vec<Lit>,
    trail_lim: Vec<i32>,
    reason: Vec<Option<ClauseIndex>>,
    level: Vec<i32>,
    root_level: i32,
}

impl Solver {
    pub fn n_vars(&self) -> usize {
        self.assigns.len()
    }

    pub fn n_assigns(&self) -> usize {
        self.trail.len()
    }

    pub fn n_clauses(&self) -> usize {
        self.clauses.len()
    }

    pub fn n_learnts(&self) -> usize {
        self.learnts.len()
    }

    pub fn value(&self, x: Var) -> LBool {
        self.assigns[x]
    }

    pub fn value_lit(&self, p: Lit) -> LBool {
        if p.sign() {
            !self.assigns[p.var()]
        } else {
            !self.assigns[p.var()]
        }
    }

    pub fn decision_level(&self) -> i32 {
        self.trail_lim.len() as i32
    }

    pub fn new_var(&mut self) -> Var {
        let v = self.n_vars();
        self.watches.push(vec![]);
        self.watches.push(vec![]);
        self.undos.push(vec![]);
        self.reason.push(None);
        self.assigns.push(LBool::Undef);
        self.level.push(-1);
        self.activity.push(0.0);
        self.varorder_new_var();
        v
    }

    fn varorder_new_var(&mut self) {}

    fn varorder_update(&mut self, _x: Var) {}

    fn varorder_update_all(&mut self) {}

    fn varorder_undo(&mut self) {}

    fn varorder_select(&mut self) -> Var {
        let mut max_i = 0;
        for i in 0..self.activity.len() {
            if self.value(i) == LBool::Undef {
                if self.value(max_i) == LBool::Undef || self.activity[i] > self.activity[max_i] {
                    max_i = i;
                }
            }
        }
        max_i
    }

    fn clause_locked(&self, ci: ClauseIndex) -> bool {
        let cl = self.get_clause_ref(ci);
        self.reason[cl.lits[0].var()] == Some(ci)
    }

    fn clause_remove(&mut self, _ci: ClauseIndex) {}

    fn clause_propagate(&mut self, ci: ClauseIndex, p: Lit) -> bool {
        match ci {
            ClauseIndex::Orig(index) => {
                // Make sure false lit at cl.lits[1]
                if self.clauses[index].lits[0] == !p {
                    self.clauses[index].lits[0] = self.clauses[index].lits[1];
                    self.clauses[index].lits[1] = !p;
                }

                // If 0th watch is true, clause is already satisfied
                if self.value_lit(self.clauses[index].lits[0]) == LBool::True {
                    // Re insert clause into watcher list
                    self.watches[p.index()].push(ci);
                    return true;
                }

                // Look for a new literal to watch
                for i in 2..self.clauses[index].lits.len() {
                    if self.value_lit(self.clauses[index].lits[i]) != LBool::False {
                        self.clauses[index].lits[1] = self.clauses[index].lits[i];
                        self.clauses[index].lits[i] = !p;
                        self.watches[(!self.clauses[index].lits[1]).index()].push(ci);
                        return true;
                    }
                }

                // Clause is unit under assignment
                self.watches[p.index()].push(ci);
                self.enqueue(self.clauses[index].lits[0], Some(ci))
            }
            ClauseIndex::Lrnt(index) => {
                // Make sure false lit at cl.lits[1]
                if self.learnts[index].lits[0] == !p {
                    self.learnts[index].lits[0] = self.learnts[index].lits[1];
                    self.learnts[index].lits[1] = !p;
                }

                // If 0th watch is true, clause is already satisfied
                if self.value_lit(self.learnts[index].lits[0]) == LBool::True {
                    // Re insert clause into watcher list
                    self.watches[p.index()].push(ci);
                    return true;
                }

                // Look for a new literal to watch
                for i in 2..self.learnts[index].lits.len() {
                    if self.value_lit(self.learnts[index].lits[i]) != LBool::False {
                        self.learnts[index].lits[1] = self.learnts[index].lits[i];
                        self.learnts[index].lits[i] = !p;
                        self.watches[(!self.learnts[index].lits[1]).index()].push(ci);
                        return true;
                    }
                }

                // Clause is unit under assignment
                self.watches[p.index()].push(ci);
                self.enqueue(self.learnts[index].lits[0], Some(ci))
            }
        }
    }

    // Only called at top level with empty prop queue
    fn clause_simplify(&mut self, ci: ClauseIndex) -> bool {
        let mut j = 0;
        let cl = self.get_clause_ref(ci);
        let mut lits = cl.lits.clone();
        for i in 0..lits.len() {
            if self.value_lit(lits[i]) == LBool::True {
                return true;
            } else if self.value_lit(lits[i]) == LBool::Undef {
                lits[j] = lits[i];
                j += 1;
            }
        }
        while lits.len() != j {
            lits.pop();
        }
        self.get_clause_mut_ref(ci).lits = lits;
        return false;
    }

    fn clause_undo(&mut self, _cl: ClauseIndex, _p: Lit) {}

    fn clause_calc_reason(&mut self, ci: ClauseIndex, p: Option<Lit>) -> Vec<Lit> {
        // Inv: p == None or p == cl.Lits[0]
        let cl = self.get_clause_ref(ci);
        let mut reason = vec![];
        for i in (if p == None { 0 } else { 1 })..cl.lits.len() {
            // Inv: self.value_lit(lits[i]) == FALSE
            reason.push(!cl.lits[i]);
        }
        self.cla_bump_activity(ci);
        return reason;
    }

    fn get_clause_ref(&self, ci: ClauseIndex) -> &Clause {
        match ci {
            ClauseIndex::Orig(ci) => &self.clauses[ci],
            ClauseIndex::Lrnt(ci) => &self.learnts[ci],
        }
    }

    fn get_clause_mut_ref(&mut self, ci: ClauseIndex) -> &mut Clause {
        match ci {
            ClauseIndex::Orig(ci) => &mut self.clauses[ci],
            ClauseIndex::Lrnt(ci) => &mut self.learnts[ci],
        }
    }

    fn clause_new(&mut self, mut ps: Vec<Lit>, learnt: bool) -> (bool, Option<ClauseIndex>) {
        if !learnt {
            // If any lit in ps is true, return true
            for &l in ps.iter() {
                if self.value_lit(l) == LBool::True {
                    return (true, None);
                }
            }

            // Remove all dups from ps
            ps.sort_by(|l, m| l.0.partial_cmp(&m.0).unwrap());
            ps.dedup();

            // If both p and !p occurs in ps, return true
            for i in 1..ps.len() {
                if ps[i - 1] == !ps[i] {
                    return (true, None);
                }
            }

            // Remove all false lits from ps
            ps = ps
                .iter()
                .map(|&l| l)
                .filter(|&l| self.value_lit(l) == LBool::False)
                .collect();
        }

        if ps.len() == 0 {
            return (false, None);
        } else if ps.len() == 1 {
            return (self.enqueue(ps[0], None), None);
        } else {
            if learnt {
                // Index of the lit with highest decision level
                let mut max_i = 0;
                for i in 0..ps.len() {
                    if self.level[ps[i].var()] > self.level[ps[max_i].var()] {
                        max_i = i;
                    }
                }

                // Pick second variable to watch
                let tmp = ps[1];
                ps[1] = ps[max_i];
                ps[max_i] = tmp;
            }

            let ci = if learnt {
                let ci = ClauseIndex::Orig(self.clauses.len());
                self.watches[(!ps[0]).index()].push(ci);
                self.watches[(!ps[1]).index()].push(ci);
                self.clauses.push(Clause { lits: ps });
                ci
            } else {
                let ci = ClauseIndex::Lrnt(self.learnts.len());
                self.watches[(!ps[0]).index()].push(ci);
                self.watches[(!ps[1]).index()].push(ci);
                for i in 0..ps.len() {
                    self.var_bump_activity(ps[i].var());
                }
                self.learnts.push(Clause { lits: ps });
                self.cla_activity.push(0.0);
                self.cla_bump_activity(ci);
                ci
            };

            return (true, Some(ci));
        }
    }

    fn var_bump_activity(&mut self, x: Var) {
        self.activity[x] += self.var_inc;
        if self.activity[x] > 1e100 {
            self.var_rescale_activity();
        }
        self.varorder_update(x);
    }

    fn var_decay_activity(&mut self) {
        self.var_inc *= self.var_decay;
    }

    fn var_rescale_activity(&mut self) {
        for i in 0..self.activity.len() {
            self.activity[i] *= 1e-100;
        }
        self.var_inc *= 1e-100;
    }

    fn cla_bump_activity(&mut self, ci: ClauseIndex) {
        if let ClauseIndex::Lrnt(index) = ci {
            self.cla_activity[index] += self.cla_inc;
            if self.cla_activity[index] > 1e100 {
                self.cla_rescale_activity();
            }
        }
    }

    fn cla_decay_activity(&mut self) {
        self.cla_inc *= self.cla_decay;
    }

    fn cla_rescale_activity(&mut self) {
        for cl in self.cla_activity.iter_mut() {
            *cl *= 1e-100;
        }
        self.cla_inc *= 1e-100;
    }

    fn decay_activities(&mut self) {
        self.var_decay_activity();
        self.cla_decay_activity();
    }

    fn propagate(&mut self) -> Option<ClauseIndex> {
        while self.prop_q.len() > 0 {
            let p = self.prop_q.pop_back().unwrap();
            let tmp = self.watches[p.index()].clone();
            self.watches[p.index()].clear();
            for i in 0..tmp.len() {
                if !self.propagate_clause(tmp[i], p) {
                    // Contraint is conflicting
                    for j in i + 1..tmp.len() {
                        self.watches[p.index()].push(tmp[j]);
                    }
                    self.prop_q.clear();
                    return Some(tmp[i]);
                }
            }
        }
        None
    }

    fn propagate_clause(&mut self, c: ClauseIndex, p: Lit) -> bool {
        unimplemented!()
    }

    fn enqueue(&mut self, p: Lit, from: Option<ClauseIndex>) -> bool {
        if self.value_lit(p) != LBool::Undef {
            if self.value_lit(p) == LBool::False {
                return false;
            } else {
                return true;
            }
        } else {
            self.assigns[p.var()] = LBool::from(!p.sign());
            self.level[p.var()] = self.decision_level();
            // self.reason[p.var()] = from;
            self.trail.push(p);
            self.prop_q.push_back(p);
            return true;
        }
    }

    fn analyze(&mut self, mut confl: ClauseIndex) -> (Vec<Lit>, i32) {
        // TODO Relook
        let mut seen = vec![false; self.n_vars()];
        let mut counter = 0;
        let mut p = None; // Undef initially TODO
        let mut p_reason = vec![];

        let mut out_learnt = vec![Lit(0)]; // Change to asserting literal, later
        let mut out_btlevel = 0;
        loop {
            p_reason.clear();
            p_reason = self.clause_calc_reason(confl, p); // Inv: confl != NULL

            for j in 0..p_reason.len() {
                let q = p_reason[j];
                if !seen[q.var()] {
                    seen[q.var()] = true;
                    if self.level[q.var()] == self.decision_level() {
                        counter += 1;
                    } else if self.level[q.var()] > 0 {
                        out_learnt.push(!q);
                        out_btlevel = if out_btlevel > self.level[q.var()] {
                            out_btlevel
                        } else {
                            self.level[q.var()]
                        };
                    }
                }
            }

            loop {
                p = self.trail.last().and_then(|&x| Some(x));
                let v = p.unwrap().var();
                confl = self.reason[v].unwrap();
                self.undo_one();
                if seen[v] {
                    break;
                }
            }
            counter -= 1;

            if counter <= 0 {
                break;
            }
        }
        (out_learnt, out_btlevel)
    }

    fn record(&mut self, clause: Vec<Lit>) {
        // TODO Fix this
        let (_, c) = self.clause_new(clause, true);
        if let Some(cl) = c {
            // self.learnts.push(cl);
        }
        // self.enqueue(clause[0], c.and_then(|_| Some(self.learnts.len()-1)));
    }

    fn undo_one(&mut self) {
        unimplemented!()
    }

    fn assume(&mut self, p: Lit) -> bool {
        self.trail_lim.push(self.trail.len() as i32);
        self.enqueue(p, None)
    }

    fn cancel(&mut self) {
        let mut c = self.trail.len() as i32 - *self.trail_lim.last().unwrap();
        while c != 0 {
            self.undo_one();
            c -= 1;
        }
        self.trail_lim.pop();
    }

    fn cancel_until(&mut self, level: i32) {
        while self.decision_level() > level {
            self.cancel()
        }
    }

    fn search(&mut self, nof_conflicts: u32, nof_learnts: u32, params: (f64, f64)) -> LBool {
        let mut conflit_c = 0;
        self.var_decay = 1.0 / params.0;
        self.cla_decay = 1.0 / params.1;
        let mut model = vec![true; self.n_vars()];

        loop {
            let confl = self.propagate();
            match confl {
                // Conflit
                Some(c) => {
                    conflit_c += 1;
                    let (learnt_clause, backtrack_level) = self.analyze(c);
                    self.cancel_until(if backtrack_level > self.root_level {
                        backtrack_level
                    } else {
                        self.root_level
                    });
                    self.record(learnt_clause);
                    self.decay_activities();
                }
                // No Conflict
                None => {
                    if self.decision_level() == 0 {
                        self.simplify_db();
                    }

                    if self.learnts.len() as i32 - self.n_assigns() as i32 >= nof_learnts as i32 {
                        self.reduce_db();
                    }

                    if self.n_assigns() == self.n_vars() {
                        // Model found
                        for i in 0..model.len() {
                            model[i] = self.value(i) == LBool::True;
                        }
                        self.cancel_until(self.root_level);
                        return LBool::True;
                    } else if conflit_c >= nof_conflicts {
                        self.cancel_until(self.root_level); // Force a restart
                        return LBool::Undef;
                    } else {
                        // New variable decision
                        let p = Lit::new(self.varorder_select(), false);
                        self.assume(p);
                    }
                }
            }
        }
    }

    fn reduce_db(&mut self) {
        let lim = self.cla_inc / self.learnts.len() as f64;

        unimplemented!();
    }

    fn simplify_db(&mut self) -> bool {
        if let Some(_) = self.propagate() {
            return false;
        }

        unimplemented!();
    }

    pub fn solve(&mut self, assumps: Vec<Lit>) -> bool {
        let params = (0.95, 0.999);
        let mut nof_conflicts = 100.0;
        let mut nof_learnts: f64 = (self.n_clauses() as f64) / 3.0;
        let mut status = LBool::Undef;

        // Push incremental assumptions
        for i in 0..assumps.len() {
            if !self.assume(assumps[i]) {
                self.cancel_until(0);
                return false;
            } else if let Some(_) = self.propagate() {
                self.cancel_until(0);
                return false;
            }
        }
        self.root_level = self.decision_level();

        // Solve
        while status == LBool::Undef {
            status = self.search(nof_conflicts as u32, nof_learnts as u32, params);
            nof_conflicts *= 1.5;
            nof_learnts *= 1.1;
        }

        self.cancel_until(0);

        return status == LBool::True;
    }
}
