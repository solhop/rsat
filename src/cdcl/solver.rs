use super::clause_db::{ClauseDb, ClauseIndex};
use super::drat_clauses::{DratClause, DratClauses};
use super::solver_options::SolverOptions;
use super::trail::Trail;
use super::VarManager;
use solhop_types::{Clause, LBool, Lit, Solution, Var, UNDEF_LIT};
use std::collections::HashSet;
use std::collections::VecDeque;

/// Represents a CDCL solver.
pub struct Solver {
    undef_state: bool,
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
    pub fn new(options: SolverOptions) -> Self {
        let clause_db = ClauseDb::new(options.clause_db_options);
        let var_manager = VarManager::new(options.branching_heuristic);
        Self {
            undef_state: false,
            clause_db,
            var_manager,
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
    fn n_assigns(&self) -> usize {
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

    /// Returns the current decision level in the solver.
    fn decision_level(&self) -> i32 {
        self.trail.decision_level()
    }

    /// Add a new variable to the solver.
    pub fn new_var(&mut self) -> Var {
        self.watches.push(vec![]);
        self.watches.push(vec![]);
        self.var_manager.new_var()
    }

    /// Add `n` new variables to the solver.
    pub fn new_vars(&mut self, n: usize) -> Vec<Var> {
        (0..n).map(|_| self.new_var()).collect()
    }

    /// Add a new clause to the solver.
    pub fn add_clause(&mut self, lits: Vec<Lit>) {
        let (r, _) = self.clause_new(lits, false);
        if !r {
            self.undef_state = true;
        }
    }

    /// Drat clauses
    pub fn drat_clauses(self) -> Option<Vec<DratClause>> {
        self.drat_clauses.drat_clauses()
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
    // Only called on learnt clause
    fn clause_simplify(&mut self, ci: ClauseIndex) -> bool {
        let mut j = 0;
        let cl = self.clause_db.get_clause_ref(ci);
        let mut lits = cl.lits.clone();
        for i in 0..lits.len() {
            if self.var_manager.value_lit(lits[i]) == LBool::True {
                return true;
            } else if self.var_manager.value_lit(lits[i]) == LBool::Undef {
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
            debug_assert!(self.var_manager.value_lit(cl.lits[i]) == LBool::False);
            reason.push(!cl.lits[i]);
        }
        self.clause_db.found_clause_as_reason(ci);
        reason
    }

    fn clause_new(&mut self, mut ps: Vec<Lit>, learnt: bool) -> (bool, Option<ClauseIndex>) {
        if !learnt {
            // If any lit in ps is true, return true
            for &l in ps.iter() {
                if self.var_manager.value_lit(l) == LBool::True {
                    return (true, None);
                }
            }

            // Remove all dups from ps
            ps.sort_by(|l, m| l.index().partial_cmp(&m.index()).unwrap());
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
                .filter(|&l| self.var_manager.value_lit(l) == LBool::Undef)
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
        if self.var_manager.value_lit(p) != LBool::Undef {
            !(self.var_manager.value_lit(p) == LBool::False)
        } else {
            self.var_manager
                .update(p.var(), LBool::from(!p.sign()), self.decision_level(), from);
            self.trail.add_at_current_dl(p);
            self.prop_q.push_back(p);
            true
        }
    }

    fn analyze(&mut self, cf: ClauseIndex) -> (Vec<Lit>, i32) {
        let mut participating_variables: Vec<Var> = vec![];
        let mut reason_variables: HashSet<Var> = HashSet::new();

        let mut confl = Some(cf);
        let mut seen = vec![false; self.n_vars()];
        let mut counter = 0;
        let mut p = None;

        let mut out_learnt = vec![UNDEF_LIT]; // Change to asserting literal, later
        let mut out_btlevel = 0;
        loop {
            debug_assert!(confl != None, "Conflit cannot be null");
            // Inv: confl != NULL
            let p_reason = self.clause_calc_reason(confl.unwrap(), p);

            // Trace reason for p
            for q in p_reason {
                if !seen[q.var().index()] {
                    participating_variables.push(q.var());
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
        if !seen[out_learnt[0].var().index()] {
            participating_variables.push(out_learnt[0].var());
        }
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

    fn search(&mut self, nof_conflicts: u32, nof_learnts: u32) -> (LBool, Vec<bool>) {
        let mut conflit_count = 0;

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
                    self.clause_db.after_record_learnt_clause();
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

    fn reduce_db(&mut self) {
        self.clause_db
            .reduce_db(&self.var_manager, &mut self.watches, &mut self.drat_clauses);
    }

    fn simplify_db(&mut self) -> bool {
        if self.propagate().is_some() {
            return false;
        }

        let cls = self.clause_db.learnt_indices();
        for i in cls {
            if self.clause_simplify(ClauseIndex::Lrnt(i)) {
                self.clause_db
                    .remove_learnt(i, &mut self.watches, &mut self.drat_clauses);
            }
        }
        true
    }

    /// Solve the SAT formula under given assumptions.
    pub fn solve(&mut self, assumps: Vec<Lit>) -> Solution {
        let solution = self.solve_(assumps);
        if let Solution::Unsat = solution {
            self.drat_clauses.capture(&[], false);
        }
        solution
    }

    fn solve_(&mut self, assumps: Vec<Lit>) -> Solution {
        if self.undef_state {
            return Solution::Unsat;
        }
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
            let res = self.search(nof_conflicts as u32, nof_learnts as u32);
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
