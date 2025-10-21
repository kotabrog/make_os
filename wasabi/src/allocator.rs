extern crate alloc;

use crate::result::Result;
use crate::uefi::{
    EfiMemoryDescriptor, EfiMemoryType, MemoryMapHolder,
};
use alloc::alloc::{
    GlobalAlloc, Layout,
};
use alloc::boxed::Box;
use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::cmp::max;
use core::fmt;
use core::mem::size_of;
use core::ops::DerefMut;
use core::ptr::null_mut;

pub fn round_up_to_nearest_pow2(v: usize) -> Result<usize> {
    1usize
        .checked_shl(usize::BITS - v.wrapping_sub(1).leading_zeros())
        .ok_or("Out of range")
}

/// Vertical bar `|` represents the chunk that has a Header.
/// before: |-- prev ----|---- self ------------
/// align: |-----|-----|------|------|------|
/// after: |----------||------|--------------
struct Header {
    next_header: Option<Box<Header>>,
    size: usize,
    is_allocated: bool,
    _reserved: usize,
}

const HEADER_SIZE: usize = size_of::<Header>();

#[allow(clippy::assertions_on_constants)]
const _:() = assert!(HEADER_SIZE == 32);

// Size of Header should be power of 2
const _:() = assert!(HEADER_SIZE.count_ones() == 1);

pub const LAYOUT_PAGE_4K: Layout = unsafe {
    Layout::from_size_align_unchecked(4096, 4096)
};

impl Header {
    fn can_provide(&self, size: usize, align: usize) -> bool {
        // This check is rough - actual size needed may be smaller.
        // HEADER_SIZE * 2 => one for allocated region, another for padding.
        self.size >= size + HEADER_SIZE * 2 + align
    }

    fn is_allocated(&self) -> bool {
        self.is_allocated
    }

    fn end_addr(&self) -> usize {
        self as *const Header as usize + self.size
    }

    unsafe fn new_from_addr(addr: usize) -> Box<Header> {
        let header = addr as *mut Header;
        header.write(Header {
            next_header: None,
            size: 0,
            is_allocated: false,
            _reserved: 0,
        });
        Box::from_raw(addr as *mut Header)
    }

    unsafe fn from_allocated_region(addr: *mut u8) -> Box<Header> {
        let header = addr.sub(HEADER_SIZE) as *mut Header;
        Box::from_raw(header)
    }

    //
    // Note: std::alloc::Layout doc says:
    // < All layouts have an associated size and power-of-two alignment.
    fn provide(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        let size = max(round_up_to_nearest_pow2(size).ok()?, HEADER_SIZE);
        let align = max(align, HEADER_SIZE);
        if self.is_allocated() || !self.can_provide(size, align) {
            None
        } else {
            // Each char represents 32-byte chunks.
            //
            // |----|-------------self-----------|--------
            // |----|-------------------       |--------
            //                          ^ self.end_addr()
            //                   |-----|-
            //                    ^ header_for_allocated
            //                     ^ allocated_addr
            //                         ^ header_for_padding
            // header_for_allocated.end_addr() self has enough space
            // to allocate the requested object.

            // Make a Header for the allocated object.
            let mut size_used = 0;
            let allocated_addr = (self.end_addr() - size) & !(align - 1);
        }
    }
}
