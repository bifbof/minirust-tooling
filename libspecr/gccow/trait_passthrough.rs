use crate::libspecr::*;

use std::fmt::{Formatter, Debug, Error};
use std::hash::{Hash, Hasher};

impl<T> Debug for GcCow<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.call_ref_unchecked(|t| t.fmt(f))
    }
}

impl<T> Hash for GcCow<T> where T: Hash {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.call_ref_unchecked(|t| t.hash(state))
    }
}

impl<T> PartialEq for GcCow<T> where T: GcCompat + PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.call_ref1_unchecked(*other, |s, o| s == o)
    }
}

impl<T> Eq for GcCow<T> where T: GcCompat + Eq {}


impl<T> Default for GcCow<T> where T: Default + GcCompat {
    fn default() -> Self {
        Self::new(T::default())
    }
}
