use crate::msat::clause_db::ClauseIndex;
use crate::*;

pub struct VarManager {
    assigns: Vec<LBool>,
    // activity: Vec<f64>,
    reason: Vec<Option<ClauseIndex>>,
    level: Vec<i32>,
    // var_inc: f64,
    // var_decay: f64,
    alpha: f64,
    learnt_counter: usize,
    ema: Vec<f64>,
    assigned: Vec<usize>,
    participated: Vec<usize>,
    reasoned: Vec<usize>,
}

impl VarManager {
    pub fn new(_var_inc: f64, _var_decay: f64) -> Self {
        VarManager {
            assigns: vec![],
            // activity: vec![],
            reason: vec![],
            level: vec![],
            // var_inc,
            // var_decay,
            alpha: 0.4,
            learnt_counter: 0,
            ema: vec![],
            assigned: vec![],
            participated: vec![],
            reasoned: vec![],
        }
    }

    pub fn n_vars(&self) -> usize {
        self.assigns.len()
    }

    pub fn new_var(&mut self) -> Var {
        let v = Var::new(self.n_vars());
        self.reason.push(None);
        self.assigns.push(LBool::Undef);
        self.level.push(-1);
        // self.activity.push(0.0);
        self.ema.push(0.0);
        self.assigned.push(0);
        self.participated.push(0);
        self.reasoned.push(0);
        v
    }

    pub fn value(&self, x: Var) -> LBool {
        self.assigns[x.index()]
    }

    pub fn value_lit(&self, p: Lit) -> LBool {
        if p.sign() {
            !self.assigns[p.var().index()]
        } else {
            self.assigns[p.var().index()]
        }
    }

    pub fn after_conflict_analysis(
        &mut self,
        participating_variables: std::collections::HashSet<Var>,
        reasoned_variables: std::collections::HashSet<Var>,
    ) {
        self.learnt_counter += 1;
        for v in participating_variables {
            self.participated[v.index()] += 1;
        }
        if self.alpha > 0.06 {
            self.alpha -= 1e-6;
        }
        for v in reasoned_variables {
            self.reasoned[v.index()] += 1;
        }
        for v in 0..self.n_vars() {
            if self.assigns[v] == LBool::Undef {
                self.ema[v] *= 0.95;
            }
        }
    }

    pub fn select_var(&self) -> Var {
        let max_v = (0..self.n_vars())
            .filter(|v| self.value(Var::new(*v)) == LBool::Undef)
            .max_by(|&x, &y| self.ema[x].partial_cmp(&self.ema[y]).unwrap())
            .unwrap();
        Var::new(max_v)
    }

    pub fn var_bump_activity(&mut self, _x: Var) {
        // self.activity[x.index()] += self.var_inc;
        // if self.activity[x.index()] > 1e100 {
        //     self.var_rescale_activity();
        // }
    }

    pub fn var_decay_activity(&mut self) {
        // self.var_inc *= self.var_decay;
    }

    // pub fn var_rescale_activity(&mut self) {
    // for i in 0..self.activity.len() {
    //     self.activity[i] *= 1e-100;
    // }
    // self.var_inc *= 1e-100;
    // }

    pub fn update_var_decay(&mut self, _var_decay: f64) {
        // self.var_decay = var_decay;
    }

    pub fn get_reason(&self, var: Var) -> Option<ClauseIndex> {
        self.reason[var.index()]
    }

    pub fn update(&mut self, var: Var, value: LBool, level: i32, reason: Option<ClauseIndex>) {
        if value != LBool::Undef {
            self.assigned[var.index()] = self.learnt_counter;
            self.participated[var.index()] = 0;
            self.reasoned[var.index()] = 0;
        } else {
            let interval = self.learnt_counter - self.assigned[var.index()];
            if interval > 0 {
                let interval = interval as f64;
                let r = self.participated[var.index()] as f64 / interval;
                let rsr = self.reasoned[var.index()] as f64 / interval;
                let prev_ema = self.ema[var.index()];
                let next_ema = (1.0 - self.alpha) * prev_ema + self.alpha * (r + rsr);
                self.ema[var.index()] = next_ema;
            }
        }
        self.assigns[var.index()] = value;
        self.level[var.index()] = level;
        self.reason[var.index()] = reason;
    }

    pub fn reset(&mut self, var: Var) {
        self.update(var, LBool::Undef, -1, None);
    }

    pub fn model(&self) -> Vec<bool> {
        self.assigns.iter().map(|&x| x == LBool::True).collect()
    }

    pub fn get_level(&self, var: Var) -> i32 {
        self.level[var.index()]
    }
}
