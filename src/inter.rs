// TODO: implement custom Deserialize for Inter
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Inter {
    // maybe structural would be better split into Sum and Product variants here rather than
    // storing the composition. that would allow sum types to have unnamed variants and be
    // transparent to traversal
    Structural(Composition, HashMap<String, Inter>),
    Nominal(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Composition {
    Sum,
    Product,
}

impl PartialOrd for Inter {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        let rhs_contains_self = self.contained_by(rhs);
        let self_contains_rhs = rhs.contained_by(self);
        match self_contains_rhs {
            true => match rhs_contains_self {
                // both are assignable to the other, so they're equal
                true => Some(std::cmp::Ordering::Equal),
                // self contains rhs, but not the converse, so self is greater
                false => Some(std::cmp::Ordering::Greater),
            },
            false => match rhs_contains_self {
                // rhs contains self but not the converse, so rhs is greater
                true => Some(std::cmp::Ordering::Less),
                // they're disjoint
                false => None,
            },
        }
    }
}

impl Inter {
    pub fn contained_by(&self, peer: &Self) -> bool {
        self.try_fit_within(peer).peekable().peek().is_none()
    }

    pub fn try_fit_within<'a>(
        &'a self,
        peer: &'a Self,
    ) -> impl Iterator<Item = Incompatibility> + 'a {
        struct IncompatibilityStream<'a> {
            to_check: VecDeque<(String, &'a Inter, Option<&'a Inter>)>,
        }

        impl<'a> Iterator for IncompatibilityStream<'a> {
            type Item = Incompatibility;
            fn next(&mut self) -> Option<Incompatibility> {
                while let Some((path, node, possible_peer)) = self.to_check.pop_front() {
                    if let Some(peer) = possible_peer {
                        match node {
                            Inter::Nominal(node_type) => match peer {
                                Inter::Nominal(peer_type) => {
                                    if node_type != peer_type {
                                        return Some(Incompatibility::MismatchedName(path));
                                    }
                                }
                                Inter::Structural(_, _) => {
                                    return Some(Incompatibility::ContainerDiverges(path))
                                }
                            },
                            Inter::Structural(node_comp, node_suc) => match peer {
                                Inter::Nominal(_) => {
                                    return Some(Incompatibility::ContainedDiverges(path))
                                }
                                Inter::Structural(peer_comp, peer_suc) => {
                                    // if they're both structural nodes, first push all the next
                                    // nodes
                                    for (key, node_val) in node_suc {
                                        let next_path = path.clone() + "." + key;
                                        self.to_check.push_back((
                                            next_path,
                                            node_val,
                                            peer_suc.get(key),
                                        ));
                                    }
                                    // then verify they compose in the same way
                                    if node_comp != peer_comp {
                                        return Some(Incompatibility::MismatchedComposition(path));
                                    }
                                }
                            },
                        }
                    } else {
                        return Some(Incompatibility::ContainedDiverges(path));
                    }
                }
                None
            }
        }

        return IncompatibilityStream {
            to_check: VecDeque::from([("".into(), self, Some(peer))]),
        };
    }
}

// in the future these may get more metadata, but for now we'll just pass a string representation
// of the path to the error
#[derive(Debug, Clone, PartialEq)]
pub enum Incompatibility {
    MismatchedName(String),
    ContainerDiverges(String),
    ContainedDiverges(String),
    MismatchedComposition(String),
}

impl Display for Incompatibility {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Incompatibility::MismatchedName(path) => write!(f, "Type mismatch at path {}", path),
            Incompatibility::ContainerDiverges(path) => write!(
                f,
                "The interfaces have different structures at path {}",
                path
            ),
            Incompatibility::ContainedDiverges(path) => {
                write!(f, "The new interface diverges from the old one at {}", path)
            }
            Incompatibility::MismatchedComposition(path) => {
                write!(f, "Composition mismatch at path {}", path)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_interfaces() {
        let a = Inter::Nominal("my-type".into());
        assert!(a.contained_by(&a));
        let b = Inter::Nominal("my-type".into());
        assert!(a.contained_by(&b));
        assert!(b.contained_by(&a));
    }

    #[test]
    fn subset() {
        let a = Inter::Structural(
            Composition::Product,
            HashMap::from([("key1".into(), Inter::Nominal("my-type".into()))]),
        );
        let b = Inter::Structural(
            Composition::Product,
            HashMap::from([
                ("key1".into(), Inter::Nominal("my-type".into())),
                ("key2".into(), Inter::Nominal("my-type".into())),
            ]),
        );
        assert!(a.contained_by(&b));

        assert_eq!(
            b.try_fit_within(&a).collect::<Vec<_>>(),
            vec![Incompatibility::ContainedDiverges(".key2".to_string())]
        );
    }

    #[test]
    fn union() {}

    #[test]
    fn intersection() {}
}
