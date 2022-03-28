use crate::Inter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// tree of identifiers designating some subset of an interface. variants ordered by descending
/// traversal size
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Loc {
    /// tip represents traversing all subsets of an interface
    Tip,
    /// structure enumerates specific subsets of the interface
    Structure(HashMap<String, Loc>),
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
            Self::Structure(self_next) => match with {
                // with is > self. use it instead
                Self::Tip => *self = with,
                // possibly disjoint, so we need to merge
                Self::Structure(with_next) => {
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
            Self::Structure(self_succ) => match base {
                Inter::Nominal(_) => None,
                Inter::Structural(comp, base_succ) => {
                    let mut succ = HashMap::new();
                    for (key, next_loc) in self_succ {
                        // loc may diverge from interface. return None if it does
                        let next_base = base_succ.get(key)?;

                        // this needs fixed. if a subset at this key diverges, we should return
                        // None from the entire func, but if we just reach an Empty loc, we should
                        // ignore the key. will either need to change return type or how Empty is
                        // handled (e.g. return an empty interface of some sort)
                        if let Some(subset) = next_loc.select_subset(next_base) {
                            succ.insert(key.clone(), subset);
                        }
                    }
                    Some(Inter::Structural(comp.clone(), succ))
                }
            },
            Self::Empty => None,
        }
    }
}
