use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::mem;
use std::num::NonZero;
use std::ptr;

use crate::ScratchObject;
use crate::data_types::ScratchCmp;

const _A: () = {
    assert!(mem::size_of::<ScratchListDummy>() == mem::size_of::<ScratchList<ScratchObject>>());
    assert!(mem::size_of::<ScratchListDummy>() == mem::size_of::<ScratchList<f64>>());
};

#[repr(C)]
pub struct ScratchListDummy {
    ptr: usize,
    len: u32,
    cap: u32,
}

impl ScratchListDummy {
    pub fn from_list<T: Clone + Default>(list: &ScratchList<T>) -> Self {
        Self {
            ptr: list.buf as usize,
            len: list.len,
            cap: list.cap.get(),
        }
    }

    pub unsafe fn clone_unsafe(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            cap: self.cap,
        }
    }
}

/// A `Vec<T>` except:
/// - 1-indexed
/// - Grows by doubling capacity, and starts at capacity 1
/// - Doesn't store garbage data: every slot in `0..cap` is always initialized
///   (so branchless bitmask-based bounds checking is safe)
/// - Out of bounds accesses are ignored or produce default values
/// - Scratch-like equality semantics
/// - Is `repr(C)` so it can be used in JIT code
///
/// # Invariants
///
/// - `buf` always valid and `cap >= 1`.
/// - Slots `0..len` contain valid `T` values.
/// - Slots `len..cap` contain valid "empty" `T` values (from `FEmpty`).
/// - `len <= cap`.
#[repr(C)]
pub struct ScratchList<T: Clone + Default> {
    buf: *mut T,
    len: u32,
    cap: NonZero<u32>,
}

impl<T: Clone + Default> Clone for ScratchList<T> {
    fn clone(&self) -> Self {
        let cap = self.cap;
        let buf = Self::alloc_buf(cap);

        unsafe {
            for i in 0..cap.get() as usize {
                buf.add(i).write((&*self.buf.add(i)).clone());
            }
        }

        Self {
            buf,
            len: self.len,
            cap,
        }
    }
}

impl<T: Clone + Default> ScratchList<T> {
    pub fn new() -> Self {
        let buf = Self::alloc_buf(NonZero::new(1).unwrap());
        unsafe { buf.write(T::default()) };
        Self {
            buf,
            len: 0,
            cap: NonZero::new(1).unwrap(),
        }
    }

    pub fn ptr(&mut self) -> *mut T {
        self.buf
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.len
    }

    #[inline]
    pub fn cap(&self) -> u32 {
        self.cap.get()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn alloc_buf(cap: NonZero<u32>) -> *mut T {
        if mem::size_of::<T>() == 0 {
            return ptr::dangling_mut();
        }
        let layout = Layout::array::<T>(cap.get() as usize).expect("ScratchList capacity overflow");
        let ptr = unsafe { alloc(layout) } as *mut T;
        if ptr.is_null() {
            handle_alloc_error(layout)
        } else {
            ptr
        }
    }

    fn dealloc_buf(buf: *mut T, cap: NonZero<u32>) {
        if mem::size_of::<T>() == 0 {
            return;
        }
        let layout = Layout::array::<T>(cap.get() as usize).expect("ScratchList capacity overflow");
        unsafe { dealloc(buf as *mut u8, layout) };
    }

    /// Grow to twice the current capacity (or 1 if currently empty).
    fn grow(&mut self) {
        let new_cap = self
            .cap
            .checked_mul(NonZero::new(2).unwrap())
            .expect("ScratchList capacity overflow");
        let new_buf = Self::alloc_buf(new_cap);
        let old_cap = self.cap.get() as usize;

        // Move existing values into new buffer.
        unsafe {
            ptr::copy_nonoverlapping(self.buf, new_buf, old_cap);
        }
        // Fill remaining slots with `empty()`.
        unsafe {
            for i in old_cap..new_cap.get() as usize {
                ptr::write(new_buf.add(i), T::default());
            }
        }
        Self::dealloc_buf(self.buf, self.cap);

        self.buf = new_buf;
        self.cap = new_cap;
    }

    /// Appends `value` to the end of the list, growing it if necessary.
    pub fn push(&mut self, value: T) {
        if self.len == self.cap.get() {
            self.grow();
        }
        unsafe {
            let slot = self.buf.add(self.len as usize);
            ptr::drop_in_place(slot); // Drop the "empty"
            ptr::write(slot, value);
        }
        self.len += 1;
    }

    /// Removes the element at `one_idx`, shifting all elements after it to the
    /// left. (1-indexed)
    ///
    /// If out of bounds, this does nothing.
    pub fn delete(&mut self, one_idx: usize) {
        if one_idx == 0 {
            return;
        }
        let idx = one_idx - 1;
        if idx >= self.len as usize {
            return;
        }
        let base = self.buf;
        let len = self.len as usize;
        unsafe {
            let empty = T::default();
            ptr::drop_in_place(base.add(idx));
            let remaining = len - idx - 1;
            if remaining > 0 {
                ptr::copy(base.add(idx + 1), base.add(idx), remaining);
            }
            ptr::write(base.add(len - 1), empty);
        }
        self.len -= 1;
    }

    /// Inserts `value` at `one_idx`, shifting all elements after it to the
    /// right. (1-indexed)
    ///
    /// If out of bounds, this does nothing.
    pub fn insert(&mut self, one_idx: usize, value: T) {
        if one_idx == 0 {
            return;
        }
        let idx = one_idx - 1;
        if idx > self.len as usize {
            return;
        }
        if self.len == self.cap.get() {
            self.grow();
        }
        let base = self.buf;
        let len = self.len as usize;
        unsafe {
            ptr::drop_in_place(base.add(len)); // Drop the "empty" at the end
            let tail = len - idx;
            if tail > 0 {
                ptr::copy(base.add(idx), base.add(idx + 1), tail);
            }
            ptr::write(base.add(idx), value);
        }
        self.len += 1;
    }

    /// Replaces the element at `one_idx` with `value`, if it exists.
    /// (1-indexed)
    ///
    /// If out of bounds, this does nothing.
    pub fn replace(&mut self, one_idx: usize, value: T) {
        // TODO: invent a fast path for this too
        // as it's a common operation
        #[deprecated]
        const _E: () = ();
        _E;

        if one_idx == 0 {
            return;
        }
        let idx = one_idx - 1;
        if idx >= self.len as usize {
            return;
        }
        let base = self.buf;
        unsafe {
            ptr::drop_in_place(base.add(idx));
            ptr::write(base.add(idx), value);
        }
    }

    /// Clears the list (does not deallocate memory).
    pub fn clear(&mut self) {
        let base = self.buf;
        unsafe {
            for i in 0..self.len as usize {
                let empty = T::default();
                ptr::drop_in_place(base.add(i));
                ptr::write(base.add(i), empty);
            }
        }
        self.len = 0;
    }

    pub fn get(&self, one_idx: usize) -> T {
        let zero_idx = one_idx.wrapping_sub(1);
        // Safety: it's non-zero
        let mask = unsafe { (self.cap.get() as usize).unchecked_sub(1) };
        let sanitized_idx = zero_idx & mask;
        let within_bounds = sanitized_idx == zero_idx;

        let live: &T = unsafe { &*self.buf.add(sanitized_idx) };
        let selected: &T = if within_bounds { live } else { &T::default() };
        selected.clone()
    }
}

impl<T: Clone + ScratchCmp + Default> ScratchList<T> {
    /// Finds the 1-indexed position of the first element equal to `value`
    /// (using the provided comparison function), or 0 if not found.
    pub fn index_of(&self, target: &T) -> usize {
        let base = self.buf;
        for i in 0..self.len as usize {
            let elem = unsafe { &*base.add(i) }; // Safety: 0..len always in bounds
            if elem.scratch_cmp(target) == std::cmp::Ordering::Equal {
                return i + 1;
            }
        }
        0
    }
}

impl<T: Clone + Default> Drop for ScratchList<T> {
    fn drop(&mut self) {
        let base = self.buf;
        for i in 0..self.cap.get() as usize {
            unsafe {
                ptr::drop_in_place(base.add(i));
            }
        }
        Self::dealloc_buf(base, self.cap);
    }
}
