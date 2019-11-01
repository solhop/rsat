use crate::errors::*;
use crate::*;
use regex::Regex;
use std::io::BufRead;

/// Dimacs formula.
pub enum Dimacs {
    /// Unweighted formula.
    Cnf {
        /// Number of variables.
        n_vars: usize,
        /// Clauses.
        clauses: Vec<Clause>,
    },
    /// Weighted formula.
    Wcnf {
        /// Number of variables.
        n_vars: usize,
        /// Clauses with their weights.
        clauses: Vec<(Clause, u64)>,
        /// Weight corresponding to hard clause.
        hard_weight: u64,
    },
}

/// Parse dimacs from buffer reader.
pub fn parse_dimacs_from_buf_reader<F>(reader: &mut F) -> Result<Dimacs>
where
    F: std::io::BufRead,
{
    let mut n_clauses = 0usize;
    let mut n_vars = 0usize;
    let mut clauses = vec![];
    let mut weights: Vec<u64> = vec![];
    let mut hard_weight = 0u64;
    let mut is_wcnf = false;

    for line in reader.lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('c') {
            continue;
        } else if line.starts_with('p') {
            let re_cnf = Regex::new(r"p\s+cnf\s+(\d+)\s+(\d+)").unwrap();
            let re_wcnf = Regex::new(r"p\s+wcnf\s+(\d+)\s+(\d+)\s+(\d+)").unwrap();
            if let Some(cap) = re_cnf.captures(&line) {
                n_vars = cap[1].parse()?;
                n_clauses = cap[2].parse()?;
            } else if let Some(cap) = re_wcnf.captures(&line) {
                is_wcnf = true;
                n_vars = cap[1].parse()?;
                n_clauses = cap[2].parse()?;
                hard_weight = cap[3].parse()?;
            }
        } else {
            let re = Regex::new(r"(-?\d+)").unwrap();
            let mut cl = vec![];
            let mut weight = 0u64;
            for (i, cap) in re.captures_iter(&line).enumerate() {
                if i == 0 && is_wcnf {
                    weight = cap[1].parse::<u64>()?;
                    continue;
                }
                let l = match cap[1].parse::<i32>()? {
                    0 => continue,
                    n => n,
                };
                let sign = if l < 0 { 1 } else { 0 };
                let var = (l.abs() - 1) as usize;
                let l = 2 * var + sign;
                cl.push(Lit(l));
            }
            clauses.push(Clause { lits: cl });
            weights.push(weight);
            if clauses.len() == n_clauses {
                break;
            }
        }
    }

    Ok(if is_wcnf {
        Dimacs::Wcnf {
            n_vars,
            clauses: clauses.into_iter().zip(weights).collect(),
            hard_weight,
        }
    } else {
        Dimacs::Cnf { n_vars, clauses }
    })
}
