use std::collections::VecDeque;
use std::ops::Not;

#[derive(Clone, Copy, PartialEq)]
pub struct Lit(pub usize);

impl Lit {
    pub fn sign(self) -> bool {
        self.0 & 1 == 1
    }

    pub fn var(self) -> usize {
        self.0 >> 1
    }

    pub fn index(self) -> usize {
        self.0
    }
}

impl Not for Lit {
    type Output = Self;

    fn not(self) -> Self {
        if self.0 % 2 == 0 {
            Lit(self.0 + 1)
        } else {
            Lit(self.0 - 1)
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum LBool {
    True,
    False,
    Undef,
}

impl Not for LBool {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            LBool::True => LBool::False,
            LBool::False => LBool::True,
            LBool::Undef => LBool::Undef,
        }
    }
}

impl From<bool> for LBool {
    fn from(b: bool) -> Self {
        if b {
            LBool::True
        } else {
            LBool::False
        }
    }
}

pub struct Clause {
    lits: Vec<Lit>,
}

#[derive(Default)]
pub struct Solver {
    clauses: Vec<Clause>,
    learnts: Vec<Clause>,
    watches: Vec<Vec<usize>>,
    assigns: Vec<LBool>,
    prop_q: VecDeque<Lit>,
}

impl Solver {
    pub fn new() -> Self {
        Solver::default()
    }

    pub fn n_vars(&self) -> usize {
        self.assigns.len()
    }

    pub fn n_clauses(&self) -> usize {
        self.clauses.len()
    }

    pub fn n_learnts(&self) -> usize {
        self.learnts.len()
    }

    pub fn value(&self, x: usize) -> LBool {
        self.assigns[x]
    }

    pub fn value_lit(&self, p: Lit) -> LBool {
        if p.sign() {
            !self.assigns[p.var()]
        } else {
            self.assigns[p.var()]
        }
    }

    pub fn new_var(&mut self) -> usize {
        let index = self.n_vars();
        self.watches.push(vec![]);
        self.watches.push(vec![]);
        self.assigns.push(LBool::Undef);
        index
    }

    pub fn new_clause(&mut self, lits: Vec<Lit>) -> bool {
        if lits.is_empty() {
            return false;
        }
        if lits.len() == 1 {
            if !self.prop_q.contains(&lits[0]) {
                self.prop_q.push_back(lits[0]);
            }
        } else {
            for &lit in lits.iter() {
                if self.watches.len() <= lit.index() {
                    return false;
                }
            }
            self.watches[lits[0].index()].push(self.clauses.len());
        }
        self.clauses.push(Clause { lits });
        true
    }

    pub fn solve(&mut self) -> bool {
        for c in self.clauses.iter() {
            if c.lits.is_empty() {
                return false;
            }
        }

        // 0 -> None tried, 1 -> F tried but not T
        // 2 -> T tried but not F, 3 -> both tried
        let mut state = vec![0; self.n_vars()];
        let mut d = 0;

        loop {
            if d == self.n_vars() {
                return true;
            }

            let mut tried = false;

            for &a in [0, 1].iter() {
                if (state[d] >> a) & 1 == 0 {
                    tried = true;
                    state[d] |= 1 << a;
                    self.assigns[d] = (a == 1).into();
                    if !self.update_watchlist(Lit(d << 1 | a)) {
                        self.assigns[d] = LBool::Undef;
                    } else {
                        d += 1;
                        break;
                    }
                }
            }

            if !tried {
                if d == 0 {
                    return false;
                } else {
                    state[d] = 0;
                    self.assigns[d] = LBool::Undef;
                    d -= 1;
                }
            }
        }
    }

    fn update_watchlist(&mut self, false_lit: Lit) -> bool {
        while !self.watches[false_lit.index()].is_empty() {
            let &cl_index = self.watches[false_lit.index()].last().unwrap();
            let mut found_alt = false;
            for &alt in self.clauses[cl_index].lits.iter() {
                let v = alt.var();
                let s = alt.sign();
                if self.assigns[v] == LBool::Undef || self.assigns[v] == s.into() {
                    found_alt = true;
                    self.watches[alt.index()].push(cl_index);
                    self.watches[false_lit.index()].pop();
                    break;
                }
            }

            if !found_alt {
                return false;
            }
        }
        true
    }
}
