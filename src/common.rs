use std::ops::Not;

/// A variable.
pub type Var = usize;

/// A literal.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Lit(pub usize);

impl Lit {
    /// Returns true if literal is signed (i.e. a negated literal).
    pub fn sign(self) -> bool {
        self.0 & 1 == 1
    }

    /// Returns the var corresponding to the literal.
    pub fn var(self) -> Var {
        self.0 >> 1
    }

    /// Returns the actual value stored inside
    /// that can be used to index arrays.
    pub fn index(self) -> usize {
        self.0
    }

    /// Create lit from var and sign
    pub fn new(var: Var, sign: bool) -> Lit {
        Lit(var + var + (sign as usize))
    }
}

impl Not for Lit {
    type Output = Self;

    /// Returns x for -x and -x for x.
    fn not(self) -> Self {
        Lit(self.0 ^ 1)
    }
}

/// A Lifted boolean.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LBool {
    /// Represents True.
    True,
    /// Represents False.
    False,
    /// Represents neither True nor False, usually used when variable is unassigned.
    Undef,
}

impl Not for LBool {
    type Output = Self;

    /// Returns True for False and False for True.
    /// If the input is Undef, then Undef is returned.
    fn not(self) -> Self {
        match self {
            LBool::True => LBool::False,
            LBool::False => LBool::True,
            LBool::Undef => LBool::Undef,
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
#[derive(Clone, Debug)]
pub struct Clause {
    /// A vector of literals forming the clause.
    pub lits: Vec<Lit>,
}

/// Solution to the SAT Formula.
#[derive(Debug)]
pub enum Solution {
    /// The formula is unsatisfiable.
    Unsat,
    /// Neither SAT or UNSAT was proven. Best model known so far.
    Best(Vec<bool>),
    /// The formula is satisfiable. A satifying model for the formula.
    Sat(Vec<bool>),
    /// No solution could be found.
    Unknown,
}

/// Errors module.
#[allow(missing_docs)]
pub mod errors {
    error_chain::error_chain! {
        foreign_links {
            ParseIntError(std::num::ParseIntError);
        }
    }
}
