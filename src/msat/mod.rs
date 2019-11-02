use std::collections::HashMap;
use std::collections::VecDeque;

use crate::{Clause, LBool, Lit, Solution, Var};

#[derive(Clone, Copy, PartialEq, Debug)]
enum ClauseIndex {
    Orig(usize),
    Lrnt(usize),
}

struct SolverOptions {
    cla_inc: f64,
    cla_decay: f64,
    var_inc: f64,
    var_decay: f64,
}

/// Different Solver Options.
pub enum SolverOption {
    /// The clause activity decay factor.
    ClaDecay(f64),
    /// The variable activity decay factor.
    VarDecay(f64),
}

impl Default for SolverOptions {
    fn default() -> Self {
        SolverOptions {
            cla_inc: 1.0,
            cla_decay: 0.999,
            var_inc: 1.0,
            var_decay: 0.95,
        }
    }
}

impl SolverOptions {
    fn option(&mut self, option: SolverOption) {
        match option {
            SolverOption::ClaDecay(v) => self.cla_decay = v,
            SolverOption::VarDecay(v) => self.var_decay = v,
        }
    }
}

#[derive(Default)]
struct ClauseDb {
    original: Vec<Clause>,
    learnts: HashMap<usize, (Clause, f64)>,
    curr_learnt_id: usize,
}

#[derive(Default)]
struct VarManager {
    assigns: Vec<LBool>,
    activity: Vec<f64>,
    reason: Vec<Option<ClauseIndex>>,
    level: Vec<i32>,
}

/// Represents a CDCL solver.
#[derive(Default)]
pub struct Solver {
    options: SolverOptions,
    clause_db: ClauseDb,
    var_manager: VarManager,
    watches: Vec<Vec<ClauseIndex>>,
    prop_q: VecDeque<Lit>,
    trail: Vec<Lit>,
    trail_lim: Vec<i32>,
    root_level: i32,
}

impl Solver {
    /// Create a new CDCL solver.
    pub fn new() -> Self {
        Solver::default()
    }

    /// Configure solver option.
    pub fn option(&mut self, option: SolverOption) {
        self.options.option(option);
    }

    /// Returns the number of variables in the formula.
    pub fn n_vars(&self) -> usize {
        self.var_manager.assigns.len()
    }

    /// Returns the number of assigned variables in the formula.
    pub fn n_assigns(&self) -> usize {
        self.trail.len()
    }

    /// Returns the number of original clauses in the formula.
    pub fn n_clauses(&self) -> usize {
        self.clause_db.original.len()
    }

    /// Returns the number of learnt clauses in the formula.
    pub fn n_learnts(&self) -> usize {
        self.clause_db.learnts.len()
    }

    /// Returns the assignment of the variable.
    pub fn value(&self, x: Var) -> LBool {
        self.var_manager.assigns[x]
    }

    /// Returns the value of the literal under current partial assignment.
    pub fn value_lit(&self, p: Lit) -> LBool {
        Self::value_lit_from_assigns(&self.var_manager.assigns, p)
    }

    fn value_lit_from_assigns(assigns: &[LBool], p: Lit) -> LBool {
        if p.sign() {
            !assigns[p.var()]
        } else {
            assigns[p.var()]
        }
    }

    /// Returns the current decision level in the solver.
    pub fn decision_level(&self) -> i32 {
        self.trail_lim.len() as i32
    }

    /// Add a new variable to the solver.
    pub fn new_var(&mut self) -> Var {
        let v = self.n_vars();
        self.watches.push(vec![]);
        self.watches.push(vec![]);
        self.var_manager.reason.push(None);
        self.var_manager.assigns.push(LBool::Undef);
        self.var_manager.level.push(-1);
        self.var_manager.activity.push(0.0);
        v
    }

    /// Add a new clause to the solver.
    pub fn new_clause(&mut self, lits: Vec<Lit>) -> bool {
        let (r, _) = self.clause_new(lits, false);
        r
    }

    fn varorder_select(&mut self) -> Var {
        let mut max_i = 0;
        for i in 0..self.var_manager.activity.len() {
            if self.value(i) == LBool::Undef
                && (self.value(max_i) != LBool::Undef
                    || self.var_manager.activity[i] > self.var_manager.activity[max_i])
            {
                max_i = i;
            }
        }
        max_i
    }

    fn clause_locked(&self, ci: ClauseIndex) -> bool {
        let cl = self.get_clause_ref(ci);
        self.var_manager.reason[cl.lits[0].var()] == Some(ci)
    }

    fn clause_remove_learnt(&mut self, ci: ClauseIndex) {
        if let ClauseIndex::Lrnt(index) = ci {
            let learnt = self.clause_db.learnts.get(&index).unwrap();
            if let Some(i) = self.watches[(!learnt.0.lits[0]).index()]
                .iter()
                .position(|&s| s == ci)
            {
                self.watches[(!learnt.0.lits[0]).index()].remove(i);
            }
            if let Some(i) = self.watches[(!learnt.0.lits[1]).index()]
                .iter()
                .position(|&s| s == ci)
            {
                self.watches[(!learnt.0.lits[1]).index()].remove(i);
            }
            self.clause_db.learnts.remove(&index);
        }
    }

    fn clause_propagate(&mut self, ci: ClauseIndex, p: Lit) -> bool {
        let enqueue_lit = match ci {
            ClauseIndex::Orig(index) => {
                // Make sure false lit at cl.lits[1]
                if self.clause_db.original[index].lits[0] == !p {
                    self.clause_db.original[index].lits[0] = self.clause_db.original[index].lits[1];
                    self.clause_db.original[index].lits[1] = !p;
                }

                // If 0th watch is true, clause is already satisfied
                if self.value_lit(self.clause_db.original[index].lits[0]) == LBool::True {
                    // Re insert clause into watcher list
                    self.watches[p.index()].push(ci);
                    return true;
                }

                // Look for a new literal to watch
                for i in 2..self.clause_db.original[index].lits.len() {
                    if self.value_lit(self.clause_db.original[index].lits[i]) != LBool::False {
                        self.clause_db.original[index].lits[1] =
                            self.clause_db.original[index].lits[i];
                        self.clause_db.original[index].lits[i] = !p;
                        self.watches[(!self.clause_db.original[index].lits[1]).index()].push(ci);
                        return true;
                    }
                }

                // Clause is unit under assignment
                self.watches[p.index()].push(ci);
                self.clause_db.original[index].lits[0]
            }
            ClauseIndex::Lrnt(index) => {
                // Make sure false lit at cl.lits[1]
                let learnt = self.clause_db.learnts.get_mut(&index).unwrap();
                if learnt.0.lits[0] == !p {
                    learnt.0.lits[0] = learnt.0.lits[1];
                    learnt.0.lits[1] = !p;
                }

                // If 0th watch is true, clause is already satisfied
                if Self::value_lit_from_assigns(&self.var_manager.assigns, learnt.0.lits[0])
                    == LBool::True
                {
                    // Re insert clause into watcher list
                    self.watches[p.index()].push(ci);
                    return true;
                }

                // Look for a new literal to watch
                for i in 2..learnt.0.lits.len() {
                    if Self::value_lit_from_assigns(&self.var_manager.assigns, learnt.0.lits[i])
                        != LBool::False
                    {
                        learnt.0.lits[1] = learnt.0.lits[i];
                        learnt.0.lits[i] = !p;
                        self.watches[(!learnt.0.lits[1]).index()].push(ci);
                        return true;
                    }
                }

                // Clause is unit under assignment
                self.watches[p.index()].push(ci);
                learnt.0.lits[0]
            }
        };
        self.enqueue(enqueue_lit, Some(ci))
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
        false
    }

    fn clause_calc_reason(&mut self, ci: ClauseIndex, p: Option<Lit>) -> Vec<Lit> {
        // Inv: p == None or p == cl.Lits[0]
        let cl = self.get_clause_ref(ci);
        debug_assert!(p == None || p == Some(cl.lits[0]));
        let mut reason = vec![];
        for i in (if p == None { 0 } else { 1 })..cl.lits.len() {
            // Inv: self.value_lit(lits[i]) == FALSE
            debug_assert!(self.value_lit(cl.lits[i]) == LBool::False);
            reason.push(!cl.lits[i]);
        }
        self.cla_bump_activity(ci);
        reason
    }

    fn get_clause_ref(&self, ci: ClauseIndex) -> &Clause {
        match ci {
            ClauseIndex::Orig(ci) => &self.clause_db.original[ci],
            ClauseIndex::Lrnt(ci) => &self.clause_db.learnts.get(&ci).unwrap().0,
        }
    }

    fn get_clause_mut_ref(&mut self, ci: ClauseIndex) -> &mut Clause {
        match ci {
            ClauseIndex::Orig(ci) => &mut self.clause_db.original[ci],
            ClauseIndex::Lrnt(ci) => &mut self.clause_db.learnts.get_mut(&ci).unwrap().0,
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
                .copied()
                .filter(|&l| self.value_lit(l) == LBool::Undef)
                .collect();
        }

        if ps.is_empty() {
            (false, None)
        } else if ps.len() == 1 {
            (self.enqueue(ps[0], None), None)
        } else {
            if learnt {
                // Index of the lit with highest decision level
                let mut max_i = 0;
                for i in 0..ps.len() {
                    if self.var_manager.level[ps[i].var()] > self.var_manager.level[ps[max_i].var()]
                    {
                        max_i = i;
                    }
                }

                // Pick second variable to watch
                ps.swap(1, max_i);
            }

            let ci = if !learnt {
                let ci = ClauseIndex::Orig(self.clause_db.original.len());
                self.watches[(!ps[0]).index()].push(ci);
                self.watches[(!ps[1]).index()].push(ci);
                self.clause_db.original.push(Clause { lits: ps });
                ci
            } else {
                let ci = ClauseIndex::Lrnt(self.clause_db.curr_learnt_id);
                self.watches[(!ps[0]).index()].push(ci);
                self.watches[(!ps[1]).index()].push(ci);
                for p in &ps {
                    self.var_bump_activity(p.var());
                }
                self.clause_db
                    .learnts
                    .insert(self.clause_db.curr_learnt_id, (Clause { lits: ps }, 0.0));
                self.clause_db.curr_learnt_id += 1;
                self.cla_bump_activity(ci);
                ci
            };

            (true, Some(ci))
        }
    }

    fn var_bump_activity(&mut self, x: Var) {
        self.var_manager.activity[x] += self.options.var_inc;
        if self.var_manager.activity[x] > 1e100 {
            self.var_rescale_activity();
        }
    }

    fn var_decay_activity(&mut self) {
        self.options.var_inc *= self.options.var_decay;
    }

    fn var_rescale_activity(&mut self) {
        for i in 0..self.var_manager.activity.len() {
            self.var_manager.activity[i] *= 1e-100;
        }
        self.options.var_inc *= 1e-100;
    }

    fn cla_bump_activity(&mut self, ci: ClauseIndex) {
        if let ClauseIndex::Lrnt(index) = ci {
            let cl = self.clause_db.learnts.get_mut(&index).unwrap();
            cl.1 += self.options.cla_inc;
            if cl.1 > 1e100 {
                self.cla_rescale_activity();
            }
        }
    }

    fn cla_decay_activity(&mut self) {
        self.options.cla_inc *= self.options.cla_decay;
    }

    fn cla_rescale_activity(&mut self) {
        for (_, cl) in self.clause_db.learnts.iter_mut() {
            cl.1 *= 1e-100;
        }
        self.options.cla_inc *= 1e-100;
    }

    fn decay_activities(&mut self) {
        self.var_decay_activity();
        self.cla_decay_activity();
    }

    fn propagate(&mut self) -> Option<ClauseIndex> {
        while !self.prop_q.is_empty() {
            let p = self.prop_q.pop_back().unwrap();
            let tmp = self.watches[p.index()].clone();
            self.watches[p.index()].clear();

            for i in 0..tmp.len() {
                if !self.clause_propagate(tmp[i], p) {
                    // Contraint is conflicting
                    for &c_i in tmp.iter().skip(i + 1) {
                        self.watches[p.index()].push(c_i);
                    }
                    self.prop_q.clear();
                    return Some(tmp[i]);
                }
            }
        }
        None
    }

    fn enqueue(&mut self, p: Lit, from: Option<ClauseIndex>) -> bool {
        if self.value_lit(p) != LBool::Undef {
            !(self.value_lit(p) == LBool::False)
        } else {
            self.var_manager.assigns[p.var()] = LBool::from(!p.sign());
            self.var_manager.level[p.var()] = self.decision_level();
            self.var_manager.reason[p.var()] = from;
            self.trail.push(p);
            self.prop_q.push_back(p);
            true
        }
    }

    fn analyze(&mut self, cf: ClauseIndex) -> (Vec<Lit>, i32) {
        let mut confl = Some(cf);
        let mut seen = vec![false; self.n_vars()];
        let mut counter = 0;
        let mut p = None;

        let mut out_learnt = vec![Lit(0)]; // Change to asserting literal, later
        let mut out_btlevel = 0;
        loop {
            debug_assert!(confl != None, "Conflit cannot be null");
            // Inv: confl != NULL
            let p_reason = self.clause_calc_reason(confl.unwrap(), p);

            // Trace reason for p
            for q in p_reason {
                if !seen[q.var()] {
                    seen[q.var()] = true;
                    if self.var_manager.level[q.var()] == self.decision_level() {
                        counter += 1;
                    } else if self.var_manager.level[q.var()] > 0 {
                        out_learnt.push(!q);
                        out_btlevel = if out_btlevel > self.var_manager.level[q.var()] {
                            out_btlevel
                        } else {
                            self.var_manager.level[q.var()]
                        };
                    }
                }
            }

            // Select next literal to look at
            loop {
                p = self.trail.last().and_then(|&x| Some(x));
                let v = p.unwrap().var();
                confl = self.var_manager.reason[v];
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
        out_learnt[0] = !(p.unwrap());
        (out_learnt, out_btlevel)
    }

    fn record(&mut self, clause: Vec<Lit>) {
        let asserting_lit = clause[0];
        let (_, c) = self.clause_new(clause, true);
        self.enqueue(asserting_lit, c);
    }

    fn undo_one(&mut self) {
        let p = self.trail.last().and_then(|&x| Some(x)).unwrap();
        let x = p.var();
        self.var_manager.assigns[x] = LBool::Undef;
        self.var_manager.reason[x] = None;
        self.var_manager.level[x] = -1;
        self.trail.pop();
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

    fn search(
        &mut self,
        nof_conflicts: u32,
        nof_learnts: u32,
        params: (f64, f64),
    ) -> (LBool, Vec<bool>) {
        let mut conflit_c = 0;
        self.options.var_decay = 1.0 / params.0;
        self.options.cla_decay = 1.0 / params.1;

        loop {
            let confl = self.propagate();
            match confl {
                // Conflit
                Some(c) => {
                    conflit_c += 1;
                    if self.decision_level() == self.root_level {
                        return (LBool::False, vec![]);
                    }
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

                    if self.clause_db.learnts.len() as i32 - self.n_assigns() as i32
                        >= nof_learnts as i32
                    {
                        self.reduce_db();
                    }

                    if self.n_assigns() == self.n_vars() {
                        // Model found
                        let model = self
                            .var_manager
                            .assigns
                            .iter()
                            .map(|&x| x == LBool::True)
                            .collect();
                        self.cancel_until(self.root_level);
                        return (LBool::True, model);
                    } else if conflit_c >= nof_conflicts {
                        // Force a restart
                        self.cancel_until(self.root_level);
                        // println!(
                        //     "c Restarting after {} conflicts, learnt {} {}, clauses {}",
                        //     conflit_c,
                        //     self.clause_db.learnts.len(),
                        //     nof_learnts,
                        //     self.clause_db.original.len()
                        // );
                        return (LBool::Undef, vec![]);
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
        let mut i = 0;
        let lim = self.options.cla_inc / self.clause_db.learnts.len() as f64;

        let mut acts: Vec<_> = self
            .clause_db
            .learnts
            .iter()
            .map(|(&i, &(_, a))| (i, a))
            .collect();
        acts.sort_unstable_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

        while i < acts.len() / 2 {
            let ci = ClauseIndex::Lrnt(acts[i].0);
            if !self.clause_locked(ci) {
                self.clause_remove_learnt(ci);
            }
            i += 1;
        }

        while i < self.clause_db.learnts.len() {
            let ci = ClauseIndex::Lrnt(acts[i].0);
            if !self.clause_locked(ci) && acts[i].1 < lim {
                self.clause_remove_learnt(ci);
            }
            i += 1;
        }
    }

    fn simplify_db(&mut self) -> bool {
        if self.propagate().is_some() {
            return false;
        }

        let cls: Vec<_> = self.clause_db.learnts.iter().map(|(&i, _)| i).collect();

        for i in cls {
            if self.clause_simplify(ClauseIndex::Lrnt(i)) {
                self.clause_remove_learnt(ClauseIndex::Lrnt(i));
            }
        }
        true
    }

    /// Solve the SAT formula under given assumptions.
    pub fn solve(&mut self, assumps: Vec<Lit>) -> Solution {
        let params = (0.95, 0.999);
        let restart_first = 100.0;
        let restart_inc = 2.0f64;
        let mut nof_learnts: f64 = (self.n_clauses() as f64) / 3.0;
        let mut status = LBool::Undef;

        // Push incremental assumptions
        for assump in assumps {
            if !self.assume(assump) || self.propagate().is_some() {
                self.cancel_until(0);
                return Solution::Unsat;
            }
        }
        self.root_level = self.decision_level();

        let mut model = vec![];

        // Solve
        let mut curr_restarts = 0;
        while status == LBool::Undef {
            let rest_base = restart_inc.powi(curr_restarts);
            let nof_conflicts = rest_base * restart_first;
            let res = self.search(nof_conflicts as u32, nof_learnts as u32, params);
            status = res.0;
            model = res.1;
            nof_learnts *= 1.1;
            curr_restarts += 1;
        }

        self.cancel_until(0);

        if status == LBool::True {
            Solution::Sat(model)
        } else {
            Solution::Unsat
        }
    }
}
