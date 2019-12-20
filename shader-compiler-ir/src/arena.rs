// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

//! Arena

use std::cell::BorrowError;
use std::cell::BorrowMutError;
use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::cell::UnsafeCell;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr;

/// reference to a value in an arena
#[repr(transparent)]
pub struct ArenaRef<'arena, T>(&'arena RefCell<T>);

impl<T> Copy for ArenaRef<'_, T> {}

impl<T> Clone for ArenaRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'arena, T> ArenaRef<'arena, T> {
    /// the identity of `self` -- the pointer to the value
    pub fn id(self) -> *mut T {
        self.0.as_ptr()
    }
    /// immutably borrow the value referenced by `self`
    pub fn borrow(self) -> Ref<'arena, T> {
        self.0.borrow()
    }
    /// try to immutably borrow the value referenced by `self`
    pub fn try_borrow(self) -> Result<Ref<'arena, T>, BorrowError> {
        self.0.try_borrow()
    }
    /// mutably borrow the value referenced by `self`
    pub fn borrow_mut(self) -> RefMut<'arena, T> {
        self.0.borrow_mut()
    }
    /// try to mutably borrow the value referenced by `self`
    pub fn try_borrow_mut(self) -> Result<RefMut<'arena, T>, BorrowMutError> {
        self.0.try_borrow_mut()
    }
}

impl<T> Hash for ArenaRef<'_, T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id().hash(hasher)
    }
}

impl<T> PartialEq for ArenaRef<'_, T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.id() == rhs.id()
    }
}

impl<T> Eq for ArenaRef<'_, T> {}

impl<T: fmt::Debug> fmt::Debug for ArenaRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Borrowed;
        impl fmt::Debug for Borrowed {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.pad("<borrowed>")
            }
        }
        let mut debug_tuple = f.debug_tuple("ArenaRef");
        match self.try_borrow() {
            Ok(value) => debug_tuple.field(&*value).finish(),
            Err(_) => debug_tuple.field(&Borrowed).finish(),
        }
    }
}

struct ArenaChunk<T> {
    values: *mut MaybeUninit<[RefCell<T>; 1]>,
    capacity: usize,
    used_count: usize,
}

impl<T> Drop for ArenaChunk<T> {
    fn drop(&mut self) {
        unsafe {
            let values_to_drop = std::slice::from_raw_parts_mut(self.values, self.used_count);
            let values_to_drop = values_to_drop
                as *mut [std::mem::MaybeUninit<[std::cell::RefCell<T>; 1]>]
                as *mut [[std::cell::RefCell<T>; 1]];
            ptr::drop_in_place(values_to_drop);
            mem::drop(Box::from_raw(std::slice::from_raw_parts_mut(
                self.values,
                self.capacity,
            )));
        }
    }
}

impl<T> ArenaChunk<T> {
    fn new(capacity: usize) -> Self {
        let values: Box<[MaybeUninit<[RefCell<T>; 1]>]> =
            (0..capacity).map(|_| MaybeUninit::uninit()).collect();
        assert_eq!(values.len(), capacity);
        Self {
            values: Box::into_raw(values) as *mut MaybeUninit<[RefCell<T>; 1]>,
            capacity,
            used_count: 0,
        }
    }
    fn try_push(&mut self, value: T) -> Result<&mut RefCell<T>, T> {
        if self.used_count >= self.capacity {
            Err(value)
        } else {
            unsafe { Ok(self.push_assuming_enough_space(value)) }
        }
    }
    unsafe fn push_assuming_enough_space(&mut self, value: T) -> &mut RefCell<T> {
        let element = &mut *self.values.add(self.used_count);
        ptr::write(element.as_mut_ptr(), [RefCell::new(value)]);
        self.used_count += 1;
        &mut (*element.as_mut_ptr())[0]
    }
}

/// arena
pub struct Arena<T> {
    chunks: UnsafeCell<Vec<ArenaChunk<T>>>,
}

impl<T> Arena<T> {
    // number of T values that fit in 1MiB; uses [T; 1] to get stride rather than size
    const MAX_CHUNK_SIZE: usize = std::mem::size_of::<[T; 1]>() / 0x1_00000;
    const SCALE_FACTOR: usize = 2;
    fn next_chunk_size(last_chunk_size: usize) -> usize {
        if let Some(next_chunk_size) = last_chunk_size.checked_mul(Self::SCALE_FACTOR) {
            if next_chunk_size >= Self::MAX_CHUNK_SIZE {
                Self::MAX_CHUNK_SIZE.max(1)
            } else {
                next_chunk_size.max(1)
            }
        } else {
            Self::MAX_CHUNK_SIZE.max(1)
        }
    }
    #[cold]
    fn alloc_in_new_chunk(&self, value: T) -> ArenaRef<T> {
        unsafe {
            let chunks = &mut *self.chunks.get();
            let next_chunk_size =
                Self::next_chunk_size(chunks.last().map_or(0, |chunk| chunk.capacity));
            chunks.push(ArenaChunk::new(next_chunk_size));
            ArenaRef(
                chunks
                    .last_mut()
                    .expect("known to be non-empty")
                    .push_assuming_enough_space(value),
            )
        }
    }
    /// allocate a new value, returning a reference to it
    pub fn alloc(&self, value: T) -> ArenaRef<T> {
        unsafe {
            let chunks = &mut *self.chunks.get();
            if let Some(last_chunk) = chunks.last_mut() {
                match last_chunk.try_push(value) {
                    Ok(retval) => ArenaRef(retval),
                    Err(value) => self.alloc_in_new_chunk(value),
                }
            } else {
                self.alloc_in_new_chunk(value)
            }
        }
    }
}
