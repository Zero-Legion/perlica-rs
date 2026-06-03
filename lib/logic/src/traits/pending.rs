use std::collections::HashSet;
use std::hash::Hash;

/// Per-table change tracker.
///
/// `K` is the natural key of the rows that get tracked (`WeaponInstId`,
/// `usize` for char/team indexes, `String` for stackable template-ids,
/// …).
#[derive(Debug, Clone, Default)]
pub struct PendingChanges<K: Eq + Hash + Clone> {
    dirty: HashSet<K>,
    removed: HashSet<K>,
}

impl<K: Eq + Hash + Clone> PendingChanges<K> {
    #[inline]
    pub fn new() -> Self {
        Self {
            dirty: HashSet::new(),
            removed: HashSet::new(),
        }
    }

    #[inline]
    pub fn mark_dirty(&mut self, key: K) {
        if !self.removed.contains(&key) {
            self.dirty.insert(key);
        }
    }

    #[inline]
    pub fn mark_removed(&mut self, key: K) {
        self.dirty.remove(&key);
        self.removed.insert(key);
    }

    #[inline]
    pub fn has_changes(&self) -> bool {
        !self.dirty.is_empty() || !self.removed.is_empty()
    }

    #[inline]
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    #[inline]
    pub fn removed_count(&self) -> usize {
        self.removed.len()
    }

    #[inline]
    pub fn dirty(&self) -> &HashSet<K> {
        &self.dirty
    }

    #[inline]
    pub fn removed(&self) -> &HashSet<K> {
        &self.removed
    }

    #[inline]
    pub fn take_snapshot(&mut self) -> PendingSnapshot<K> {
        PendingSnapshot {
            dirty: std::mem::take(&mut self.dirty),
            removed: std::mem::take(&mut self.removed),
        }
    }

    pub fn restore_snapshot(&mut self, snap: PendingSnapshot<K>) {
        for k in snap.dirty {
            self.mark_dirty(k);
        }
        for k in snap.removed {
            self.mark_removed(k);
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.dirty.clear();
        self.removed.clear();
    }
}

#[derive(Debug, Clone)]
pub struct PendingSnapshot<K: Eq + Hash + Clone> {
    pub dirty: HashSet<K>,
    pub removed: HashSet<K>,
}

impl<K: Eq + Hash + Clone> PendingSnapshot<K> {
    #[inline]
    pub fn has_changes(&self) -> bool {
        !self.dirty.is_empty() || !self.removed.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_then_removed_wins() {
        let mut p: PendingChanges<u64> = PendingChanges::new();
        p.mark_dirty(7);
        p.mark_removed(7);
        assert!(p.dirty().is_empty());
        assert!(p.removed().contains(&7));
    }

    #[test]
    fn dirty_after_removed_is_noop() {
        let mut p: PendingChanges<u64> = PendingChanges::new();
        p.mark_removed(3);
        p.mark_dirty(3);
        assert!(p.dirty().is_empty());
        assert!(p.removed().contains(&3));
    }

    #[test]
    fn take_snapshot_clears() {
        let mut p: PendingChanges<u64> = PendingChanges::new();
        p.mark_dirty(1);
        p.mark_dirty(2);
        p.mark_removed(9);
        let snap = p.take_snapshot();
        assert!(!p.has_changes());
        assert_eq!(snap.dirty.len(), 2);
        assert_eq!(snap.removed.len(), 1);
    }

    #[test]
    fn restore_after_failed_commit() {
        let mut p: PendingChanges<u64> = PendingChanges::new();
        p.mark_dirty(1);
        let snap = p.take_snapshot();
        // … flush failed …
        p.mark_dirty(2); // another change came in while we were trying
        p.restore_snapshot(snap);
        assert!(p.dirty().contains(&1));
        assert!(p.dirty().contains(&2));
    }
}
