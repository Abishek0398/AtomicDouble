#![warn(rust_2018_idioms)]

#![feature(stdsimd)]

pub use core::sync::atomic::{fence, Ordering};

use std::panic::RefUnwindSafe;

#[cfg(feature = "fallback")]
mod fallback;
mod ops;

use core::cell::UnsafeCell;
use core::fmt;

#[repr(C,align(16))]
pub struct AtomicDouble<T> {
    v : UnsafeCell<T>
}

unsafe impl<T: Copy + Send> Sync for AtomicDouble<T> {}

impl<T: Copy + RefUnwindSafe> RefUnwindSafe for AtomicDouble<T> {}

impl<T: Copy + Default> Default for AtomicDouble<T> {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Copy + fmt::Debug> fmt::Debug for AtomicDouble<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AtomicDouble")
            .field(&self.load(Ordering::SeqCst))
            .finish()
    }
}

impl<T> AtomicDouble<T> {
    /// Creates a new `Atomic`.
    #[inline]
    pub const fn new(v: T) -> AtomicDouble<T> {
        AtomicDouble {
            v: UnsafeCell::new(v),
        }
    }

    /// Checks if `Atomic` objects of this type are lock-free.
    ///
    /// If an `Atomic` is not lock-free then it may be implemented using locks
    /// internally, which makes it unsuitable for some situations (such as
    /// communicating with a signal handler).
    #[inline]
    pub fn is_lock_free() -> bool {
        ops::atomic_is_lock_free::<T>()
    }
}

impl<T: Copy> AtomicDouble<T> {
    /// Returns a mutable reference to the underlying type.
    ///
    /// This is safe because the mutable reference guarantees that no other threads are
    /// concurrently accessing the atomic data.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.v.get() }
    }

    /// Consumes the atomic and returns the contained value.
    ///
    /// This is safe because passing `self` by value guarantees that no other threads are
    /// concurrently accessing the atomic data.
    #[inline]
    pub fn into_inner(self) -> T {
        self.v.into_inner()
    }

    /// Loads a value from the `Atomic`.
    ///
    /// `load` takes an `Ordering` argument which describes the memory ordering
    /// of this operation.
    ///
    /// # Panics
    ///
    /// Panics if `order` is `Release` or `AcqRel`.
    #[inline]
    pub fn load(&self, order: Ordering) -> T {
        unsafe { ops::atomic_load(self.v.get(), order) }
    }

    /// Stores a value into the `Atomic`.
    ///
    /// `store` takes an `Ordering` argument which describes the memory ordering
    /// of this operation.
    ///
    /// # Panics
    ///
    /// Panics if `order` is `Acquire` or `AcqRel`.
    #[inline]
    pub fn store(&self, val: T, order: Ordering) {
        unsafe {
            ops::atomic_store(self.v.get(), val, order);
        }
    }


    /// Stores a value into the `Atomic` if the current value is the same as the
    /// `current` value.
    ///
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value. On success this value is
    /// guaranteed to be equal to `new`.
    ///
    /// `compare_exchange` takes two `Ordering` arguments to describe the memory
    /// ordering of this operation. The first describes the required ordering if
    /// the operation succeeds while the second describes the required ordering
    /// when the operation fails. The failure ordering can't be `Acquire` or
    /// `AcqRel` and must be equivalent or weaker than the success ordering.
    #[inline]
    pub fn compare_exchange(
        &self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        unsafe { ops::atomic_compare_exchange(self.v.get(), current, new, success, failure) }
    }

    /// Add to the current value, returning the previous value.
    #[inline]
    pub fn fetch_add(&self, val: T, order: Ordering) -> T {
        unsafe { ops::atomic_add(self.v.get(), val, order) }
    }

    /// Subtract from the current value, returning the previous value.
    #[inline]
    pub fn fetch_sub(&self, val: T, order: Ordering) -> T {
        unsafe { ops::atomic_sub(self.v.get(), val, order) }
    }
}