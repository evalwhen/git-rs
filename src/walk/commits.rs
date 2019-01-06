use std::collections::{ HashMap, HashSet, BinaryHeap };

use crate::objects::commit::Commit;
use crate::stores::StorageSet;
use crate::objects::Object;
use crate::id::Id;

#[derive(Debug)]
pub struct IdCommit(Id, Commit);

impl std::cmp::Ord for IdCommit {
    fn cmp(&self, other: &IdCommit) -> std::cmp::Ordering {
        if let Some(ref rhs) = self.1.committer() {
            if let Some(ref lhs) = other.1.committer() {
                return rhs.at().cmp(lhs.at());
            }
        }

        return std::cmp::Ordering::Equal;
    }
}

impl std::cmp::PartialOrd for IdCommit {
    fn partial_cmp(&self, other: &IdCommit) -> Option<std::cmp::Ordering> {
        return Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for IdCommit {
    fn eq(&self, other: &IdCommit) -> bool {
        return self.cmp(other) == std::cmp::Ordering::Equal;
    }
}

impl std::cmp::Eq for IdCommit { }

pub struct CommitIterator<'a> {
    storage_set: &'a StorageSet,
    seen: HashSet<Id>,
    target: BinaryHeap<IdCommit>
}

impl<'a> CommitIterator<'a> {
    pub fn new(storage_set: &'a StorageSet, id: &Id, seen: Option<HashSet<Id>>) -> CommitIterator<'a> {
        let mut seen = seen.unwrap_or_else(|| HashSet::<Id>::new());

        let first = storage_set.get_and_load(id).ok()
            .unwrap_or(None);

        let mut target = BinaryHeap::with_capacity(4);

        if let Some(xs) = first {
            if let Object::Commit(head) = xs {
                target.push(IdCommit(id.clone(), head));
            }
        }

        seen.insert(id.clone());
        CommitIterator {
            target,
            storage_set,
            seen,
        }
    }
}

impl<'a> Iterator for CommitIterator<'a> {
    type Item = (Id, Commit);

    fn next(&mut self) -> Option<Self::Item> {
        // okay, so given a set of commits:
        // if the set is empty, return None
        //      take() the latest commit.
        //      get the parents of the latest commit.
        //          remove all seen commits from the parents.
        //          add the remaining parent ids to seen.
        //          push remaining parent commits into the vector.

        let newest = self.target.pop()?;

        if let Some(xs) = newest.1.parents() {
            let mut seen = &mut self.seen;
            let storage_set = &self.storage_set;
            let mut parents = xs.iter().filter_map(|id| {
                if seen.contains(id) {
                    return None
                }

                if let Object::Commit(commit) = storage_set.get_and_load(id).ok()?? {
                    seen.insert(id.clone());
                    return Some(IdCommit(id.clone(), commit))
                } else {
                    return None
                }
            });

            for parent in parents {
                self.target.push(parent);
            }
        }

        return Some((newest.0, newest.1));
    }
}
