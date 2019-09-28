#![allow(dead_code)]

use std::collections::VecDeque;

use crate::common::{LBool, Lit, Var};

pub struct Clause {
    learnt: bool,
    activity: f64,
    lits: Vec<Lit>,
}

pub struct VarOrder<'a> {
    ref_to_assigns: &'a Vec<LBool>,
    ref_to_activity: &'a Vec<f64>,
}

impl VarOrder<'_> {
    pub fn new_var(&self) {
        unimplemented!()
    }

    pub fn update(&self, x: Var) {
        unimplemented!()
    }

    pub fn update_all(&self) {
        unimplemented!()
    }

    pub fn undo(&self, x: Var) {
        unimplemented!()
    }

    pub fn select(&self) -> Var {
        unimplemented!()
    }
}

pub struct Solver<'a> {
    clauses: Vec<Clause>,
    learnts: Vec<Clause>,
    cla_inc: f64,
    cla_decay: f64,
    var_inc: f64,
    var_decay: f64,
    activity: Vec<f64>,
    order: VarOrder<'a>,
    watches: Vec<Vec<Clause>>,
    undos: Vec<Vec<Clause>>,
    prop_q: VecDeque<Lit>,
    assigns: Vec<LBool>,
    trail: Vec<Lit>,
    trail_lim: Vec<i32>,
    reason: Vec<usize>,
    level: Vec<i32>,
    root_level: i32,
}

impl Solver<'_> {
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

    pub fn clause_new(&mut self, mut ps: Vec<Lit>, learnt: bool) -> (bool, Option<Clause>) {
        if !learnt {
            // If any lit in ps is true, return true
            for &l in ps.iter() {
                if self.value_lit(l) == LBool::True {
                    return (true, None);
                }
            }
            // TODO: If both p and !p occurs in ps, return true
            // Remove all false lits from ps
            ps = ps
                .iter()
                .map(|&l| l)
                .filter(|&l| self.value_lit(l) == LBool::False)
                .collect();
            // Remove all dups from ps
            ps.sort_by(|l, m| l.0.partial_cmp(&m.0).unwrap());
            ps.dedup();
        }

        if ps.len() == 0 {
            return (false, None);
        } else if ps.len() == 1 {
            return (self.enqueue(ps[0]), None);
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

                unimplemented!();
            }

            // TODO: Add to watches list

            return (
                true,
                Some(Clause {
                    learnt: learnt,
                    activity: 0.0,
                    lits: ps,
                }),
            );
        }
    }

    pub fn var_bump_activity(&mut self, x: Var) {
        self.activity[x] += self.var_inc;
        if self.activity[x] > 1e100 {
            self.var_rescale_activity();
        }
        self.order.update(x);
    }

    pub fn var_decay_activity(&mut self) {
        self.var_inc *= self.var_decay;
    }

    pub fn var_rescale_activity(&mut self) {
        for i in 0..self.activity.len() {
            self.activity[i] *= 1e-100;
        }
        self.var_inc *= 1e-100;
    }

    pub fn cla_bump_activity(&mut self, c: &mut Clause) {
        c.activity += self.cla_inc;
        if c.activity > 1e100 {
            self.cla_rescale_activity();
        }
    }

    pub fn cla_decay_activity(&mut self) {
        self.cla_inc *= self.cla_decay;
    }

    pub fn cla_rescale_activity(&mut self) {
        for cl in self.learnts.iter_mut() {
            cl.activity *= 1e-100;
        }
        self.cla_inc *= 1e-100;
    }

    pub fn decay_activities(&mut self) {
        self.var_decay_activity();
        self.cla_decay_activity();
    }

    fn propagate(&mut self) -> Option<Clause> {
        unimplemented!()
    }

    fn enqueue(&mut self, p: Lit) -> bool {
        unimplemented!()
    }

    fn analyze(confl: Clause) -> (Vec<Lit>, i32) {
        unimplemented!();
    }

    fn record(&mut self, clause: Vec<Lit>) {
        unimplemented!();
    }

    fn undo_one(&mut self) {
        unimplemented!()
    }

    fn assume(&mut self, p: Lit) -> bool {
        self.trail_lim.push(self.trail.len() as i32);
        self.enqueue(p)
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
        unimplemented!()
    }

    pub fn solve(&mut self, assumps: Vec<Lit>) -> bool {
        let params = (0.95, 0.999);
        let mut nof_conflicts = 100.0;
        let mut nof_learnts: f64 = (self.n_clauses() as f64) / 3.0;
        let mut status = LBool::Undef;

        // Push incremental assumptions
        for i in 0..assumps.len() {
            if !self.assume(assumps[i]) || false
            /*self.propagate() != None*/
            {
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
