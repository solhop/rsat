use crate::msat::clause_db::ClauseIndex;
use crate::*;

pub struct VarManager {
    assigns: Vec<LBool>,
    activity: Vec<f64>,
    reason: Vec<Option<ClauseIndex>>,
    level: Vec<i32>,
    var_inc: f64,
    var_decay: f64,
}

impl VarManager {
    pub fn new(var_inc: f64, var_decay: f64) -> Self {
        VarManager {
            assigns: vec![],
            activity: vec![],
            reason: vec![],
            level: vec![],
            var_inc,
            var_decay,
        }
    }

    pub fn n_vars(&self) -> usize {
        self.assigns.len()
    }

    pub fn new_var(&mut self) -> Var {
        self.reason.push(None);
        self.assigns.push(LBool::Undef);
        self.level.push(-1);
        self.activity.push(0.0);
        self.n_vars()
    }

    pub fn value(&self, x: Var) -> LBool {
        self.assigns[x]
    }

    pub fn value_lit(&self, p: Lit) -> LBool {
        if p.sign() {
            !self.assigns[p.var()]
        } else {
            self.assigns[p.var()]
        }
    }

    pub fn select_var(&self) -> Var {
        let mut max_i = 0;
        for i in 0..self.activity.len() {
            if self.value(i) == LBool::Undef
                && (self.value(max_i) != LBool::Undef || self.activity[i] > self.activity[max_i])
            {
                max_i = i;
            }
        }
        max_i
    }

    pub fn var_bump_activity(&mut self, x: Var) {
        self.activity[x] += self.var_inc;
        if self.activity[x] > 1e100 {
            self.var_rescale_activity();
        }
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

    pub fn update_var_decay(&mut self, var_decay: f64) {
        self.var_decay = var_decay;
    }

    pub fn update_assign(&mut self, var: Var, value: LBool) {
        self.assigns[var] = value;
    }

    pub fn get_reason(&self, var: Var) -> Option<ClauseIndex> {
        self.reason[var]
    }

    pub fn update_reason(&mut self, var: Var, reason: Option<ClauseIndex>) {
        self.reason[var] = reason;
    }

    pub fn model(&self) -> Vec<bool> {
        self.assigns.iter().map(|&x| x == LBool::True).collect()
    }

    pub fn get_level(&self, var: Var) -> i32 {
        self.level[var]
    }

    pub fn update_level(&mut self, var: Var, level: i32) {
        self.level[var] = level;
    }
}
