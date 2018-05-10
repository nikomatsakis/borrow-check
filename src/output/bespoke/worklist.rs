use fxhash::FxHashSet;
use std::hash::Hash;

crate struct WorkList<T> {
    // FIXME. This could be made more efficient if we specialized to
    // the fact that T is indexable; the "set" would just be a bit vec
    // or whatever.
    data: Vec<T>,
    set: FxHashSet<T>,
}

impl<T: Copy + Eq + Hash> WorkList<T> {
    crate fn new() -> Self {
        WorkList {
            data: Vec::default(),
            set: FxHashSet::default(),
        }
    }

    crate fn add(&mut self, value: T) {
        if self.set.insert(value) {
            self.data.push(value);
        }
    }

    crate fn next(&mut self) -> Option<T> {
        self.data.pop().map(|v| {
            self.set.remove(&v);
            v
        })
    }

    crate fn extend(&mut self, items: impl IntoIterator<Item = T>) {
        for item in items {
            self.add(item);
        }
    }
}
