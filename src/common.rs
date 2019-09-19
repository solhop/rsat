use std::ops::Not;

/// A literal.
#[derive(Clone, Copy, PartialEq)]
pub struct Lit(pub usize);

impl Lit {
    /// Returns true if literal is signed (i.e. a negated literal).
    pub fn sign(self) -> bool {
        self.0 & 1 == 1
    }

    /// Returns the var cooressponding to the literal.
    pub fn var(self) -> usize {
        self.0 >> 1
    }

    /// Returns the actual value stored inside
    /// that can be used to index arrays.
    pub fn index(self) -> usize {
        self.0
    }
}

impl Not for Lit {
    type Output = Self;

    /// Returns x for -x and -x for x.
    fn not(self) -> Self {
        if self.0 % 2 == 0 {
            Lit(self.0 + 1)
        } else {
            Lit(self.0 - 1)
        }
    }
}

/// A Lifted boolean.
#[derive(Clone, Copy, PartialEq)]
pub enum LBool {
    True,
    False,
    None,
}

impl Not for LBool {
    type Output = Self;

    /// Returns True for False and False for True.
    /// If the input is None, then None is returned.
    fn not(self) -> Self {
        match self {
            LBool::True => LBool::False,
            LBool::False => LBool::True,
            LBool::None => LBool::None,
        }
    }
}

impl From<bool> for LBool {
    /// Convert bool to LBool.
    fn from(b: bool) -> Self {
        if b {
            LBool::True
        } else {
            LBool::False
        }
    }
}

/// A Clause.
pub struct Clause(pub Vec<Lit>);

/// Solution to the SAT Formula.
#[derive(Debug)]
pub enum Solution {
    /// The formula is unsatisfiable
    Unsat,
    /// Neither SAT or UNSAT was proven. Best model known so far.
    Best(Vec<bool>),
    /// The formula is satisfiable. A satifying model for the formula.
    Sat(Vec<bool>),
}
