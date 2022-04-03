use crate::Inter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// tree of identifiers designating some subset of an interface. variants ordered by descending
/// traversal size
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Loc {
    /// tip represents traversing all subsets of an interface
    Tip,
    /// branch enumerates named subsets of the interface
    Branch(HashMap<String, Loc>),
    /// empty represents that we don't traverse this interface
    Empty,
}

impl Loc {
    pub fn new() -> Self {
        Self::Empty
    }

    pub fn consume(self: &mut Self, with: Self) {
        match self {
            // with is <= self. noop
            Self::Tip => (),
            Self::Branch(self_next) => match with {
                // with is > self. use it instead
                Self::Tip => *self = with,
                // possibly disjoint, so we need to merge
                Self::Branch(with_next) => {
                    for (key, with_val) in with_next {
                        if let Some(self_val) = self_next.get_mut(&key) {
                            self_val.consume(with_val);
                        } else {
                            self_next.insert(key, with_val);
                        }
                    }
                }
                // with is < self. noop
                Self::Empty => (),
            },
            // with is >= self. use it instead
            Self::Empty => *self = with,
        }
    }

    /// combines a loc with an interface
    pub fn select_subset(&self, base: &Inter) -> Option<Inter> {
        match self {
            Self::Tip => Some(base.clone()),
            Self::Branch(self_succ) => match base {
                // the loc diverges
                Inter::Nominal(_) => None,
                Inter::Product(base_succ) => {
                    let mut succ = HashMap::new();
                    for (key, next_loc) in self_succ {
                        // loc may diverge from interface here or deeper into this branch.
                        // propogate None either way
                        let next_base = base_succ.get(key)?;
                        let subset = next_loc.select_subset(next_base)?;
                        succ.insert(key.clone(), subset);
                    }
                    Some(Inter::Product(succ))
                }
                // this is the only place we need to match against sum. empty doesn't go deeper,
                // and tip does, but traverses all paths anyway
                Inter::Sum(base_succ) => {
                    let mut succ = Vec::new();
                    // TODO: think about better ways to represent sum traversal in Loc type
                    for member in base_succ {
                        if let Some(s) = Self::select_subset(&self, member) {
                            succ.push(s);
                        };
                    }
                    Some(Inter::Sum(succ))
                }
                // the loc diverges
                Inter::Never => None,
            },
            Self::Empty => Some(Inter::Never),
        }
    }
}
