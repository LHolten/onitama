use std::{hash::Hash, ptr};

use bigint::U256;

const EMPTY_BITS: usize = 256 - 3 * 3 * 3 * 3 * 3;
pub const BDD_ALL: U256 = U256([u64::MAX, u64::MAX, u64::MAX, u64::MAX >> EMPTY_BITS]);
pub const BDD_NONE: U256 = U256([0; 4]);

pub struct ByRef<'a, T: ?Sized>(pub &'a T);

impl<'a, T> Clone for ByRef<'a, T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<'a, T> Copy for ByRef<'a, T> {}

impl<'a, 'b, T, Y> PartialEq<ByRef<'b, Y>> for ByRef<'a, T> {
    fn eq(&self, other: &ByRef<'b, Y>) -> bool {
        ptr::eq(self.0, other.0 as *const Y as *const T)
    }
}

impl<'a, T> Eq for ByRef<'a, T> {}

impl<'a, T> Hash for ByRef<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ptr::hash(self.0, state);
    }
}

impl<'a, T> PartialOrd for ByRef<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T> Ord for ByRef<'a, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0 as *const T).cmp(&(other.0 as *const T))
    }
}

pub struct Entry<'a, T> {
    pub next: ByRef<'a, T>,
    pub cond: ByRef<'a, U256>,
}

impl<'a, T> Clone for Entry<'a, T> {
    fn clone(&self) -> Self {
        Self {
            next: self.next,
            cond: self.cond,
        }
    }
}

impl<'a, T> Copy for Entry<'a, T> {}

impl<'a, T> PartialEq for Entry<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.next == other.next && self.cond == other.cond
    }
}

impl<'a, T> Eq for Entry<'a, T> {}

impl<'a, T> Hash for Entry<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.next.hash(state);
        self.cond.hash(state);
    }
}

#[repr(transparent)]
pub struct Sdd<'a, T>(Entry<'a, T>);

impl<'a, T> Sdd<'a, T> {
    pub fn new(inner: &'a [Entry<'a, T>]) -> &'a Self {
        let mut total = BDD_NONE;
        for item in inner {
            assert!(total != BDD_ALL);
            total = total | *item.cond.0;
        }
        assert!(total == BDD_ALL);
        let raw = &inner[0] as *const Entry<'a, T> as *const Sdd<'a, T>;
        unsafe { &*raw }
    }
}

pub struct EntryIter<'a, T> {
    item: Option<&'a Entry<'a, T>>,
    total: U256,
}

impl<'a, T> Iterator for EntryIter<'a, T> {
    type Item = &'a Entry<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.item?;
        self.total = self.total | *item.cond.0;
        if self.total != BDD_ALL {
            let raw = item as *const Entry<'a, T>;
            unsafe { self.item = Some(&*raw.offset(1)) }
        } else {
            self.item = None
        }
        Some(item)
    }
}

impl<'a, T> IntoIterator for &'a Sdd<'a, T> {
    type Item = &'a Entry<'a, T>;

    type IntoIter = EntryIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        EntryIter {
            item: Some(&self.0),
            total: BDD_NONE,
        }
    }
}

pub trait Never<'a>: 'a {
    const NEVER: &'a Self;
}

impl<'a> Never<'a> for U256 {
    const NEVER: &'a Self = &BDD_NONE;
}

impl<'a, T: Never<'a>> Never<'a> for Sdd<'a, T> {
    const NEVER: &'a Self = &Sdd(Entry {
        next: ByRef(T::NEVER),
        cond: ByRef(&BDD_ALL),
    });
}
