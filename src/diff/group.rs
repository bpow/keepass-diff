use crate::diff::entry::Entry;
use crate::diff::{Diff, DiffResult, DiffResultFormat};

use keepass::Node;
use std::collections::HashMap;

/// Corresponds to a sorted Vec of KdbxEntry objects that can be diffed
#[derive(Debug)]
pub struct Group {
    name: String,
    child_groups: HashMap<String, Vec<Group>>,
    entries: HashMap<String, Vec<Entry>>,
    use_verbose: bool,
}

impl Group {
    /// Create an entries list from a keepass::Group
    pub fn from_keepass(group: &keepass::Group, use_verbose: bool) -> Self {
        let name = group.name.to_owned();

        let mut child_groups: HashMap<String, Vec<Group>> = HashMap::new();
        for node in group.children.iter() {
            match node {
                Node::Group(g) => child_groups
                    .entry(g.name.clone())
                    .or_insert(Vec::new())
                    .push(Group::from_keepass(g, use_verbose)),
                _ => {}
            }
        }

        let mut entries: HashMap<String, Vec<Entry>> = HashMap::new();
        for node in group.children.iter() {
            match node {
                Node::Entry(e) => entries
                    .entry(e.get("Title").unwrap_or_default().to_owned())
                    .or_insert(Vec::new())
                    .push(Entry::from_keepass(e, use_verbose)),
                _ => {}
            }
        }

        Group {
            name,
            child_groups,
            entries,
            use_verbose,
        }
    }
}

impl std::fmt::Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.use_verbose {
            write!(f, "Group '{}'", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

/// Groups can be diffed.
impl Diff for Group {
    fn diff<'a>(&'a self, other: &'a Group) -> DiffResult<'a, Self> {
        let (has_differences_groups, acc_groups) =
            crate::diff::diff_hashmap(&self.child_groups, &other.child_groups);

        let (has_differences_entries, acc_entries) =
            crate::diff::diff_hashmap(&self.entries, &other.entries);

        if has_differences_groups || has_differences_entries {
            let mut inner_differences: Vec<Box<dyn DiffResultFormat>> = Vec::new();

            for dr in acc_groups {
                inner_differences.push(Box::new(dr));
            }

            for dr in acc_entries {
                inner_differences.push(Box::new(dr));
            }

            DiffResult::InnerDifferences {
                left: self,
                right: other,
                inner_differences,
            }
        } else {
            DiffResult::Identical {
                left: self,
                right: other,
            }
        }
    }
}
