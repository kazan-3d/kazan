// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
use crate::util;
use errno;
use libc;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::os::raw::c_int;
use std::ptr::null_mut;

#[derive(Debug)]
#[allow(dead_code)]
pub struct SharedMemorySegment {
    id: c_int,
    size: usize,
    _phantom_data: PhantomData<*mut u8>,
}

unsafe impl Send for SharedMemorySegment {}
unsafe impl Sync for SharedMemorySegment {}

impl SharedMemorySegment {
    #[allow(dead_code)]
    pub unsafe fn new(id: c_int, size: usize) -> Self {
        assert_ne!(size, 0);
        assert_ne!(id, -1);
        SharedMemorySegment {
            id,
            size,
            _phantom_data: PhantomData,
        }
    }
    pub unsafe fn create_with_flags(size: usize, flags: c_int) -> Result<Self, errno::Errno> {
        match libc::shmget(libc::IPC_PRIVATE, size, flags) {
            -1 => Err(errno::errno()),
            id => Ok(Self::new(id, size)),
        }
    }
    #[allow(dead_code)]
    pub fn create(size: usize) -> Result<Self, errno::Errno> {
        unsafe { Self::create_with_flags(size, libc::IPC_CREAT | libc::IPC_EXCL | 0o666) }
    }
    #[allow(dead_code)]
    pub fn map(&self) -> Result<MappedSharedMemorySegment, errno::Errno> {
        unsafe {
            let memory = libc::shmat(self.id, null_mut(), 0);
            if memory == !0usize as *mut _ {
                Err(errno::errno())
            } else {
                Ok(MappedSharedMemorySegment {
                    memory: memory as *mut u8,
                    size: self.size,
                })
            }
        }
    }
}

impl Drop for SharedMemorySegment {
    fn drop(&mut self) {
        unsafe {
            libc::shmctl(self.id, libc::IPC_RMID, null_mut());
        }
    }
}

#[derive(Debug)]
pub struct MappedSharedMemorySegment {
    memory: *mut u8,
    size: usize,
}

impl MappedSharedMemorySegment {
    unsafe fn get(&self) -> *mut [u8] {
        util::to_slice_mut(self.memory, self.size)
    }
}

unsafe impl Send for MappedSharedMemorySegment {}
unsafe impl Sync for MappedSharedMemorySegment {}

impl Deref for MappedSharedMemorySegment {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe { &*self.get() }
    }
}

impl DerefMut for MappedSharedMemorySegment {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { &mut *self.get() }
    }
}

impl Drop for MappedSharedMemorySegment {
    fn drop(&mut self) {
        unsafe {
            libc::shmdt(self.memory as *const _);
        }
    }
}
