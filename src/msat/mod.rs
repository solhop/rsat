mod clause_db;
mod trail;
mod var_manager;

use crate::*;
use clause_db::{ClauseDb, ClauseIndex};
use std::collections::VecDeque;
use trail::Trail;
use var_manager::VarManager;

/// Branching heuristic to be used for msat
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

/// Solver options.
pub struct SolverOptions {
    cla_inc: f64,
    cla_decay: f64,
    branching_heuristic: BranchingHeuristic,
    // var_inc: f64,
    // var_decay: f64,
    capture_drat: bool,
}

/// Different Solver Options.
pub enum SolverOption {
    /// The clause activity decay factor.
    ClaDecay(f64),
    /// The branching heuristic to be used.
    BranchingHeuristic(BranchingHeuristic),
    /// The variable activity decay factor.
    // VarDecay(f64),
    /// Should capture conflict clauses for drat output,
    CaptureDrat,
}

impl Default for SolverOptions {
    fn default() -> Self {
        SolverOptions {
            cla_inc: 1.0,
            cla_decay: 0.999,
            branching_heuristic: BranchingHeuristic::Lrb,
            // var_inc: 1.0,
            // var_decay: 0.95,
            capture_drat: false,
        }
    }
}

impl SolverOptions {
    /// Add solver option.
    pub fn option(&mut self, option: SolverOption) {
        match option {
            SolverOption::ClaDecay(v) => self.cla_decay = v,
            // SolverOption::VarDecay(v) => self.var_decay = v,
            SolverOption::BranchingHeuristic(bh) => self.branching_heuristic = bh,
            SolverOption::CaptureDrat => self.capture_drat = true,
        }
    }
}

struct DratClauses {
    drat_clauses: Vec<(Vec<Lit>, bool)>,
    capture_drat: bool,
}

impl DratClauses {
    fn new(capture_drat: bool) -> Self {
        Self {
            drat_clauses: vec![],
            capture_drat,
        }
    }

    fn capture(&mut self, lits: &Vec<Lit>, is_delete: bool) {
        if self.capture_drat {
            self.drat_clauses.push((lits.clone(), is_delete));
        }
    }
}

/// Represents a CDCL solver.
pub struct Solver {
    clause_db: ClauseDb,
    var_manager: VarManager,
    watches: Vec<Vec<ClauseIndex>>,
    prop_q: VecDeque<Lit>,
    trail: Trail,
    root_level: i32,
    drat_clauses: DratClauses,
}

impl Solver {
    /// Create a new CDCL solver.
    /// Set drat callback which takes (lits, is_delete)
    pub fn new(options: SolverOptions) -> Self {
        Self {
            clause_db: ClauseDb::new(options.cla_inc, options.cla_decay),
            var_manager: VarManager::new(options.branching_heuristic),
            watches: vec![],
            prop_q: VecDeque::new(),
            trail: Trail::new(),
            root_level: 0,
            drat_clauses: DratClauses::new(options.capture_drat),
        }
    }

    /// Returns the number of variables in the formula.
    pub fn n_vars(&self) -> usize {
        self.var_manager.n_vars()
    }

    /// Returns the number of assigned variables in the formula.
    pub fn n_assigns(&self) -> usize {
        self.trail.n_assigns()
    }

    /// Returns the number of original clauses in the formula.
    pub fn n_clauses(&self) -> usize {
        self.clause_db.original_len()
    }

    /// Returns the number of learnt clauses in the formula.
    pub fn n_learnts(&self) -> usize {
        self.clause_db.learnts_len()
    }

    /// Returns the assignment of the variable.
    pub fn value(&self, x: Var) -> LBool {
        self.var_manager.value(x)
    }

    /// Returns the value of the literal under current partial assignment.
    pub fn value_lit(&self, p: Lit) -> LBool {
        self.var_manager.value_lit(p)
    }

    /// Returns the current decision level in the solver.
    pub fn decision_level(&self) -> i32 {
        self.trail.decision_level()
    }

    /// Add a new variable to the solver.
    pub fn new_var(&mut self) -> Var {
        self.watches.push(vec![]);
        self.watches.push(vec![]);
        self.var_manager.new_var()
    }

    /// Add a new clause to the solver.
    pub fn new_clause(&mut self, lits: Vec<Lit>) -> bool {
        let (r, _) = self.clause_new(lits, false);
        if !r {
            // In case new clause returns false, formula is unsat and solver is in undef state
            self.drat_clauses.capture(&vec![], false);
        }
        r
    }

    /// Drat clauses
    pub fn drat_clauses(self) -> Vec<(Vec<Lit>, bool)> {
        if self.drat_clauses.capture_drat {
            self.drat_clauses.drat_clauses
        } else {
            vec![]
        }
    }

    /// If the clause is reason for some variable
    /// (INVARIANT: if it is, then it should be var corresponding to first literal),
    /// then the clause is locked.
    fn is_clause_locked(&self, ci: ClauseIndex) -> bool {
        let cl = self.clause_db.get_clause_ref(ci);
        self.var_manager.get_reason(cl.lits[0].var()) == Some(ci)
    }

    /// Assume p is true and simplify the clause
    fn clause_propagate(&mut self, ci: ClauseIndex, p: Lit) -> bool {
        let clause = match ci {
            ClauseIndex::Orig(index) => self.clause_db.get_original_mut(index).unwrap(),
            ClauseIndex::Lrnt(index) => self.clause_db.get_learnt_mut(index).unwrap(),
        };

        // Make sure false lit at cl.lits[1]
        if clause.lits[0] == !p {
            clause.lits[0] = clause.lits[1];
            clause.lits[1] = !p;
        }

        // If 0th watch is true, clause is already satisfied
        if self.var_manager.value_lit(clause.lits[0]) == LBool::True {
            // Re insert clause into watcher list
            self.watches[p.index()].push(ci);
            return true;
        }

        // Look for a new literal to watch
        for i in 2..clause.lits.len() {
            if self.var_manager.value_lit(clause.lits[i]) != LBool::False {
                clause.lits[1] = clause.lits[i];
                clause.lits[i] = !p;
                self.watches[(!clause.lits[1]).index()].push(ci);
                return true;
            }
        }

        // Clause is unit under assignment
        self.watches[p.index()].push(ci);
        let enqueue_lit = clause.lits[0];
        self.enqueue(enqueue_lit, Some(ci))
    }

    // Only called at top level with empty prop queue
    fn clause_simplify(&mut self, ci: ClauseIndex) -> bool {
        let mut j = 0;
        let cl = self.clause_db.get_clause_ref(ci);
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
        self.clause_db.get_clause_mut_ref(ci).lits = lits;
        false
    }

    fn clause_calc_reason(&mut self, ci: ClauseIndex, p: Option<Lit>) -> Vec<Lit> {
        // Inv: p == None or p == cl.Lits[0]
        let cl = self.clause_db.get_clause_ref(ci);
        debug_assert!(p == None || p == Some(cl.lits[0]));
        let mut reason = vec![];
        for i in (if p == None { 0 } else { 1 })..cl.lits.len() {
            // Inv: self.value_lit(lits[i]) == FALSE
            debug_assert!(self.value_lit(cl.lits[i]) == LBool::False);
            reason.push(!cl.lits[i]);
        }
        self.clause_db.cla_bump_activity(ci);
        reason
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
                    if self.var_manager.get_level(ps[i].var())
                        > self.var_manager.get_level(ps[max_i].var())
                    {
                        max_i = i;
                    }
                }

                // Pick second variable to watch
                ps.swap(1, max_i);
            }

            let ci = if !learnt {
                let ps_0 = ps[0];
                let ps_1 = ps[1];
                let ci = self.clause_db.add_original(Clause { lits: ps });
                self.watches[(!ps_0).index()].push(ci);
                self.watches[(!ps_1).index()].push(ci);
                ci
            } else {
                self.var_manager.after_learnt_clause(&ps);
                let ps_0 = ps[0];
                let ps_1 = ps[1];
                let ci = self.clause_db.add_learnt(Clause { lits: ps });
                self.watches[(!ps_0).index()].push(ci);
                self.watches[(!ps_1).index()].push(ci);
                self.clause_db.cla_bump_activity(ci);
                ci
            };

            (true, Some(ci))
        }
    }

    /// Propagate unit clauses in prop_q and return when a confliting clause is found
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

            // TODO: There is some bug in below code or this should replace lines
            // from let tmp = ...
            // till end of for loop
            // while !self.watches[p.index()].is_empty() {
            //     let cl = self.watches[p.index()].pop().unwrap();
            //     if !self.clause_propagate(cl, p) {
            //         self.prop_q.clear();
            //         return Some(cl);
            //     }
            // }
        }
        None
    }

    fn enqueue(&mut self, p: Lit, from: Option<ClauseIndex>) -> bool {
        if self.value_lit(p) != LBool::Undef {
            !(self.value_lit(p) == LBool::False)
        } else {
            self.var_manager
                .update(p.var(), LBool::from(!p.sign()), self.decision_level(), from);
            self.trail.add_at_current_dl(p);
            self.prop_q.push_back(p);
            true
        }
    }

    fn analyze(&mut self, cf: ClauseIndex) -> (Vec<Lit>, i32) {
        use std::collections::HashSet;
        let mut participating_variables: HashSet<Var> = HashSet::new();
        let mut reason_variables: HashSet<Var> = HashSet::new();

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
                participating_variables.insert(q.var());
                if !seen[q.var().index()] {
                    seen[q.var().index()] = true;
                    if self.var_manager.get_level(q.var()) == self.decision_level() {
                        counter += 1;
                    } else if self.var_manager.get_level(q.var()) > 0 {
                        out_learnt.push(!q);
                        out_btlevel = if out_btlevel > self.var_manager.get_level(q.var()) {
                            out_btlevel
                        } else {
                            self.var_manager.get_level(q.var())
                        };
                    }
                }
            }

            // Select next literal to look at
            loop {
                p = self.trail.pop();
                let v = p.unwrap().var();
                confl = self.var_manager.get_reason(v);
                self.var_manager.reset(v);
                if seen[v.index()] {
                    break;
                }
            }
            counter -= 1;

            if counter <= 0 {
                break;
            }
        }
        out_learnt[0] = !(p.unwrap());
        participating_variables.insert(out_learnt[0].var());
        for lit in out_learnt.iter() {
            if let Some(ci) = self.var_manager.get_reason(lit.var()) {
                let clause = self.clause_db.get_clause_ref(ci);
                for lit in clause.lits.iter() {
                    reason_variables.insert(lit.var());
                }
            }
        }
        for lit in out_learnt.iter() {
            reason_variables.remove(&lit.var());
        }
        self.var_manager
            .after_conflict_analysis(participating_variables, reason_variables);
        (out_learnt, out_btlevel)
    }

    fn record(&mut self, clause: Vec<Lit>) {
        // Added here because clause_new doesn't add unit clauses to clause_db
        self.drat_clauses.capture(&clause, false);
        let asserting_lit = clause[0];
        let (_, c) = self.clause_new(clause, true);
        self.enqueue(asserting_lit, c);
    }

    fn assume(&mut self, p: Lit) -> bool {
        self.trail.new_dl();
        self.enqueue(p, None)
    }

    fn cancel(&mut self) {
        let mut c = self.trail.trail_len() as i32 - self.trail.trail_lim_pop().unwrap();
        while c != 0 {
            let p = self.trail.pop().unwrap();
            self.var_manager.reset(p.var());
            c -= 1;
        }
    }

    fn cancel_until(&mut self, level: i32) {
        while self.trail.decision_level() > level {
            self.cancel();
        }
    }

    fn search(
        &mut self,
        nof_conflicts: u32,
        nof_learnts: u32,
        decay_params: (f64, f64),
    ) -> (LBool, Vec<bool>) {
        let mut conflit_count = 0;
        self.var_manager.update_var_decay(1.0 / decay_params.0);
        self.clause_db.update_cla_decay(1.0 / decay_params.1);

        loop {
            let confl = self.propagate();
            match confl {
                // Conflit
                Some(c) => {
                    conflit_count += 1;
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
                    self.var_manager.after_record_learnt_clause();
                    self.clause_db.cla_decay_activity();
                }
                // No Conflict
                None => {
                    if self.decision_level() == 0 {
                        self.simplify_db();
                    }

                    if self.clause_db.learnts_len() as i32 - self.n_assigns() as i32
                        >= nof_learnts as i32
                    {
                        self.reduce_db();
                    }

                    if self.n_assigns() == self.n_vars() {
                        // Model found
                        let model = self.var_manager.model();
                        self.cancel_until(self.root_level);
                        return (LBool::True, model);
                    } else if conflit_count >= nof_conflicts {
                        // Force a restart
                        self.cancel_until(self.root_level);

                        return (LBool::Undef, vec![]);
                    } else {
                        // New variable decision
                        let p = Lit::new(self.var_manager.select_var(), false);
                        self.assume(p);
                    }
                }
            }
        }
    }

    fn remove_learnt_clause(&mut self, ci: ClauseIndex) {
        if let ClauseIndex::Lrnt(index) = ci {
            let learnt = self.clause_db.get_learnt(index).unwrap();
            if let Some(i) = self.watches[(!learnt.lits[0]).index()]
                .iter()
                .position(|&s| s == ci)
            {
                self.watches[(!learnt.lits[0]).index()].remove(i);
            }
            if let Some(i) = self.watches[(!learnt.lits[1]).index()]
                .iter()
                .position(|&s| s == ci)
            {
                self.watches[(!learnt.lits[1]).index()].remove(i);
            }
            self.drat_clauses.capture(&learnt.lits, true);
            self.clause_db.remove_learnt(index);
        }
    }

    fn reduce_db(&mut self) {
        let mut i = 0;
        let lim = self.clause_db.get_cla_inc() / self.clause_db.learnts_len() as f64;

        let mut acts = self.clause_db.learnt_activities();
        // Using clause length does help (TODO)
        // acts.sort_by(|(_, a1, l1), (_, a2, l2)| match l2.cmp(l1) {
        //     std::cmp::Ordering::Less => std::cmp::Ordering::Less,
        //     std::cmp::Ordering::Equal => a1.partial_cmp(a2).unwrap(),
        //     std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        // });
        acts.sort_by(|(_, a1, _), (_, a2, _)| a1.partial_cmp(a2).unwrap());

        while i < acts.len() / 2 {
            let ci = ClauseIndex::Lrnt(acts[i].0);
            if !self.is_clause_locked(ci) {
                self.remove_learnt_clause(ci);
            }
            i += 1;
        }

        while i < self.clause_db.learnts_len() {
            let ci = ClauseIndex::Lrnt(acts[i].0);
            if !self.is_clause_locked(ci) && acts[i].1 < lim {
                self.remove_learnt_clause(ci);
            }
            i += 1;
        }
    }

    fn simplify_db(&mut self) -> bool {
        if self.propagate().is_some() {
            return false;
        }

        let cls = self.clause_db.learnt_indices();
        for i in cls {
            if self.clause_simplify(ClauseIndex::Lrnt(i)) {
                self.remove_learnt_clause(ClauseIndex::Lrnt(i));
            }
        }
        true
    }

    /// Solve the SAT formula under given assumptions.
    pub fn solve(&mut self, assumps: Vec<Lit>) -> Solution {
        let solution = self.solve_(assumps);
        if let Solution::Unsat = solution {
            self.drat_clauses.capture(&vec![], false);
        }
        solution
    }

    fn solve_(&mut self, assumps: Vec<Lit>) -> Solution {
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
