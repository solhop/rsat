use crate::*;

pub struct Trail {
    trail: Vec<Lit>,
    trail_lim: Vec<i32>,
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

    pub fn cancel_until(&mut self, level: i32) -> Vec<Lit> {
        let mut cancelled = vec![];
        while self.decision_level() > level {
            let dl_index = *self.trail_lim.last().unwrap();
            while self.trail.len() as i32 != dl_index {
                cancelled.push(self.trail.pop().unwrap());
            }
            self.trail_lim.pop();
        }
        cancelled
    }
}
