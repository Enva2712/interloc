// TODO: implement custom Deserialize for Inter
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};

// TODO: ideally there would be some way to define assignability rules, but this makes no effort to
// support that
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Inter {
    /// A product type containing named members
    Product(HashMap<String, Inter>),
    /// A sum type containing a list of variants
    Sum(Vec<Inter>),
    /// A nominal type representing a named primitive
    Nominal(String),
    /// The bottom type
    Never,
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
        // this struct may need an additional vecdeq for buffering incompatibilities when a single
        // node would produce multiple. this would simplify the Product and Sum branches below, and
        // let us remove the option wrapper in to_check
        struct IncompatibilityStream<'a> {
            to_check: VecDeque<(String, &'a Inter, Option<&'a Inter>)>,
        }

        impl<'a> Iterator for IncompatibilityStream<'a> {
            type Item = Incompatibility;
            fn next(&mut self) -> Option<Incompatibility> {
                while let Some((path, node, possible_peer)) = self.to_check.pop_front() {
                    if let Some(peer) = possible_peer {
                        match peer {
                            Inter::Nominal(peer_type) => match node {
                                Inter::Nominal(node_type) => {
                                    if node_type != peer_type {
                                        return Some(Incompatibility::MismatchedName(path));
                                    }
                                }
                                _ => return Some(Incompatibility::ContainerDiverges(path)),
                            },
                            Inter::Product(peer_succ) => match node {
                                Inter::Product(node_succ) => {
                                    // if they're both structural nodes, push all the next nodes
                                    for (key, node_val) in node_succ {
                                        let next_path = path.clone() + "." + key;
                                        self.to_check.push_back((
                                            next_path,
                                            node_val,
                                            peer_succ.get(key),
                                        ));
                                    }
                                }
                                _ => return Some(Incompatibility::ContainedDiverges(path)),
                            },
                            Inter::Sum(peer_succ) => match node {
                                Inter::Sum(node_succ) => {
                                    for s in node_succ {
                                        self.to_check.push_back((path.clone(), s, Some(peer)))
                                    }
                                }
                                _ => {
                                    let mut compatible = false;
                                    for s in peer_succ {
                                        // TODO: forward incompatibilities back up
                                        if node.contained_by(s) {
                                            compatible = true;
                                            break;
                                        }
                                    }
                                    if !compatible {
                                        return Some(Incompatibility::ContainerDiverges(path));
                                    }
                                }
                            },
                            // FIXME
                            Inter::Never => {}
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
    // TODO: these two don't really make sense. clean up the variants
    ContainerDiverges(String),
    ContainedDiverges(String),
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
        let a = Inter::Product(HashMap::from([(
            "key1".into(),
            Inter::Nominal("my-type".into()),
        )]));
        let b = Inter::Product(HashMap::from([
            ("key1".into(), Inter::Nominal("my-type".into())),
            ("key2".into(), Inter::Nominal("my-type".into())),
        ]));
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
