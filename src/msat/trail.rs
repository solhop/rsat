use crate::*;

pub struct Trail {
    pub trail: Vec<Lit>,
    pub trail_lim: Vec<i32>,
}

impl Trail {
    pub fn new() -> Self {
        Trail {
            trail: vec![],
            trail_lim: vec![],
        }
    }

    pub fn n_assigns(&self) -> usize {
        self.trail.len()
    }

    pub fn decision_level(&self) -> i32 {
        self.trail_lim.len() as i32
    }

    pub fn add_at_current_dl(&mut self, p: Lit) {
        self.trail.push(p);
    }

    pub fn new_dl(&mut self) {
        self.trail_lim.push(self.trail.len() as i32);
    }

    pub fn pop(&mut self) -> Option<Lit> {
        self.trail.pop()
    }
}
