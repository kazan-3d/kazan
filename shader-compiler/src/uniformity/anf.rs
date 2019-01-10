// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use super::variable_set::VariableSet;
use spirv_parser::IdRef;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::fmt;
use std::iter;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

/// [Algebraic Normal Form](https://en.wikipedia.org/wiki/Algebraic_normal_form)
/// the empty `VariableSet` means true
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ANF {
    terms: HashSet<VariableSet>,
    variable_counts: HashMap<IdRef, usize>,
}

impl ANF {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_term(term: VariableSet) -> Self {
        let variable_counts = term.iter().map(|variable| (variable, 1)).collect();
        Self {
            terms: iter::once(term).collect(),
            variable_counts,
        }
    }
    pub fn with_terms(terms: HashSet<VariableSet>) -> Self {
        let mut variable_counts = HashMap::new();
        for term in terms.iter() {
            for variable in term.iter() {
                *variable_counts.entry(variable).or_insert(0) += 1;
            }
        }
        Self {
            terms,
            variable_counts,
        }
    }
    pub fn terms(&self) -> &HashSet<VariableSet> {
        &self.terms
    }
    pub fn variable_counts(&self) -> &HashMap<IdRef, usize> {
        &self.variable_counts
    }
    pub fn contains_term(&self, term: &VariableSet) -> bool {
        self.terms.contains(term)
    }
    pub fn insert_term(&mut self, term: &VariableSet) -> bool {
        if self.terms.insert(term.clone()) {
            for variable in term.iter() {
                *self.variable_counts.entry(variable).or_insert(0) += 1;
            }
            true
        } else {
            false
        }
    }
    pub fn remove_term(&mut self, term: &VariableSet) -> bool {
        if self.terms.remove(term) {
            for variable in term.iter() {
                if let Entry::Occupied(mut entry) = self.variable_counts.entry(variable) {
                    *entry.get_mut() -= 1;
                    if *entry.get() == 0 {
                        entry.remove();
                    }
                } else {
                    unreachable!("missing variable count for {}", variable);
                }
            }
            true
        } else {
            false
        }
    }
    /// returns true if the term is inserted
    pub fn toggle_term(&mut self, term: &VariableSet) -> bool {
        if !self.remove_term(term) {
            assert!(self.insert_term(term));
            true
        } else {
            false
        }
    }
    pub fn variable_count(&self, variable: IdRef) -> usize {
        self.variable_counts.get(&variable).cloned().unwrap_or(0)
    }
    pub fn contains_variable(&self, variable: IdRef) -> bool {
        self.variable_count(variable) != 0
    }
    pub fn not_assign(&mut self) {
        self.toggle_term(&VariableSet::new());
    }
    fn and_or(&self, rhs: &Self, is_or: bool) -> Self {
        // for `bitor` we use a | b == !(!a & !b)
        let mut retval = ANF::from(is_or);
        let mut rhs_needs_empty = is_or;
        for rhs_term in rhs.terms.iter() {
            if rhs_needs_empty && rhs_term.is_empty() {
                rhs_needs_empty = false;
                continue;
            }
            let mut lhs_needs_empty = is_or;
            for lhs_term in self.terms.iter() {
                if lhs_needs_empty && lhs_term.is_empty() {
                    lhs_needs_empty = false;
                    continue;
                }
                let merged_term = lhs_term | rhs_term;
                retval.toggle_term(&merged_term);
            }
            if lhs_needs_empty {
                retval.toggle_term(rhs_term);
            }
        }
        if rhs_needs_empty {
            retval.bitxor_assign(self);
            retval.toggle_term(&VariableSet::new());
        }
        retval
    }
}

impl Default for ANF {
    fn default() -> Self {
        Self {
            terms: HashSet::new(),
            variable_counts: HashMap::new(),
        }
    }
}

impl From<bool> for ANF {
    fn from(v: bool) -> Self {
        if v {
            Self {
                terms: iter::once(VariableSet::new()).collect(),
                variable_counts: HashMap::new(),
            }
        } else {
            Self::default()
        }
    }
}

impl From<IdRef> for ANF {
    fn from(v: IdRef) -> Self {
        let mut retval = Self::new();
        retval.insert_term(&VariableSet::from(v));
        retval
    }
}

impl fmt::Display for ANF {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut terms: Vec<&VariableSet> = self.terms.iter().collect();
        terms.sort_unstable();
        let mut terms = terms.iter();
        fn write_term(f: &mut fmt::Formatter, term: &VariableSet) -> fmt::Result {
            let mut iter = term.iter();
            let separator = if f.alternate() { '·' } else { '&' };
            if let Some(variable) = iter.next() {
                write!(f, "{}", variable)?;
                for variable in iter {
                    write!(f, " {} {}", separator, variable)?;
                }
                Ok(())
            } else if f.alternate() {
                write!(f, "T")
            } else {
                write!(f, "1")
            }
        }
        if let Some(term) = terms.next() {
            write_term(f, term)?;
            let separator = if f.alternate() { '⊕' } else { '^' };
            for term in terms {
                write!(f, "  {}  ", separator)?;
                write_term(f, term)?;
            }
            Ok(())
        } else if f.alternate() {
            write!(f, "F")
        } else {
            write!(f, "0")
        }
    }
}

impl Not for &'_ ANF {
    type Output = ANF;
    fn not(self) -> ANF {
        let mut retval = self.clone();
        retval.not_assign();
        retval
    }
}

impl Not for ANF {
    type Output = Self;
    fn not(mut self) -> Self {
        self.not_assign();
        self
    }
}

impl BitXorAssign<&'_ Self> for ANF {
    fn bitxor_assign(&mut self, rhs: &Self) {
        for term in rhs.terms.iter() {
            self.toggle_term(term);
        }
    }
}

impl BitXorAssign for ANF {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.bitxor_assign(&rhs);
    }
}

impl BitXor for &'_ ANF {
    type Output = ANF;
    fn bitxor(self, rhs: Self) -> ANF {
        let mut retval = self.clone();
        retval.bitxor_assign(rhs);
        retval
    }
}

impl BitXor<&'_ ANF> for ANF {
    type Output = ANF;
    fn bitxor(mut self, rhs: &ANF) -> ANF {
        self.bitxor_assign(rhs);
        self
    }
}

impl BitXor<ANF> for &'_ ANF {
    type Output = ANF;
    fn bitxor(self, mut rhs: ANF) -> ANF {
        rhs.bitxor_assign(self);
        rhs
    }
}

impl BitXor for ANF {
    type Output = ANF;
    fn bitxor(mut self, rhs: Self) -> ANF {
        self.bitxor_assign(rhs);
        self
    }
}

impl BitAnd for &'_ ANF {
    type Output = ANF;
    fn bitand(self, rhs: &ANF) -> ANF {
        self.and_or(rhs, false)
    }
}

impl BitAnd<&'_ ANF> for ANF {
    type Output = ANF;
    fn bitand(self, rhs: &ANF) -> ANF {
        self.and_or(rhs, false)
    }
}

impl BitAnd<ANF> for &'_ ANF {
    type Output = ANF;
    fn bitand(self, rhs: ANF) -> ANF {
        self.and_or(&rhs, false)
    }
}

impl BitAnd for ANF {
    type Output = ANF;
    fn bitand(self, rhs: ANF) -> ANF {
        self.and_or(&rhs, false)
    }
}

impl BitAndAssign<&'_ ANF> for ANF {
    fn bitand_assign(&mut self, rhs: &'_ ANF) {
        *self = (&*self).bitand(rhs);
    }
}

impl BitAndAssign for ANF {
    fn bitand_assign(&mut self, rhs: ANF) {
        self.bitand_assign(&rhs);
    }
}

impl BitOr for &'_ ANF {
    type Output = ANF;
    fn bitor(self, rhs: &ANF) -> ANF {
        self.and_or(rhs, true)
    }
}

impl BitOr<&'_ ANF> for ANF {
    type Output = ANF;
    fn bitor(self, rhs: &ANF) -> ANF {
        self.and_or(rhs, true)
    }
}

impl BitOr<ANF> for &'_ ANF {
    type Output = ANF;
    fn bitor(self, rhs: ANF) -> ANF {
        self.and_or(&rhs, true)
    }
}

impl BitOr for ANF {
    type Output = ANF;
    fn bitor(self, rhs: ANF) -> ANF {
        self.and_or(&rhs, true)
    }
}

impl BitOrAssign<&'_ ANF> for ANF {
    fn bitor_assign(&mut self, rhs: &'_ ANF) {
        *self = (&*self).bitor(rhs);
    }
}

impl BitOrAssign for ANF {
    fn bitor_assign(&mut self, rhs: ANF) {
        self.bitor_assign(&rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anf_not() {
        assert_eq!(!ANF::from(false), ANF::from(true));
        assert_eq!(!ANF::from(true), ANF::from(false));
    }

    #[test]
    fn test_anf_xor() {
        assert_eq!(ANF::from(false) ^ ANF::from(false), ANF::from(false));
        assert_eq!(ANF::from(true) ^ ANF::from(false), ANF::from(true));
        assert_eq!(ANF::from(false) ^ ANF::from(true), ANF::from(true));
        assert_eq!(ANF::from(true) ^ ANF::from(true), ANF::from(false));
    }

    #[test]
    fn test_anf_and() {
        assert_eq!(ANF::from(false) & ANF::from(false), ANF::from(false));
        assert_eq!(ANF::from(true) & ANF::from(false), ANF::from(false));
        assert_eq!(ANF::from(false) & ANF::from(true), ANF::from(false));
        assert_eq!(ANF::from(true) & ANF::from(true), ANF::from(true));
    }

    #[test]
    fn test_anf_or() {
        assert_eq!(ANF::from(false) | ANF::from(false), ANF::from(false));
        assert_eq!(ANF::from(true) | ANF::from(false), ANF::from(true));
        assert_eq!(ANF::from(false) | ANF::from(true), ANF::from(true));
        assert_eq!(ANF::from(true) | ANF::from(true), ANF::from(true));

        assert_eq!(!ANF::from(IdRef(1)) | ANF::from(IdRef(1)), ANF::from(true));

        assert_eq!(
            ANF::from(IdRef(1)) | (ANF::from(IdRef(1)) ^ ANF::from(IdRef(2))),
            ANF::with_terms(
                [
                    VariableSet::from(IdRef(1)),
                    VariableSet::from(IdRef(2)),
                    [IdRef(1), IdRef(2)].iter().cloned().collect()
                ]
                .iter()
                .cloned()
                .collect()
            )
        );
    }

    #[test]
    fn test_anf_display() {
        fn check_display(expr: ANF, normal_text: &str, alt_text: &str) {
            assert_eq!(format!("{}", expr), normal_text);
            assert_eq!(format!("{:#}", expr), alt_text);
        }

        check_display(ANF::from(false), "0", "F");
        check_display(ANF::from(true), "1", "T");
        check_display(ANF::from(IdRef(1)), "%1", "%1");
        check_display(!ANF::from(IdRef(1)), "1  ^  %1", "T  ⊕  %1");
        check_display(
            ANF::from(IdRef(1)) & ANF::from(IdRef(2)),
            "%1 & %2",
            "%1 · %2",
        );
        check_display(
            ANF::from(IdRef(1)) | ANF::from(IdRef(2)),
            "%1  ^  %1 & %2  ^  %2",
            "%1  ⊕  %1 · %2  ⊕  %2",
        );
        check_display(
            ANF::from(IdRef(1)) ^ ANF::from(IdRef(2)),
            "%1  ^  %2",
            "%1  ⊕  %2",
        );
    }
}
