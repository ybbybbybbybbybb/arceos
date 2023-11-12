use crate::{AllocError, AllocResult, BaseAllocator, PageAllocator, ByteAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;

struct PageInfo {
    pos: usize,
    used_pages: usize,
}

struct ByteInfo {
    pos: usize,
    pos_byte: usize,
    used_pages: usize,
    used_bytes: usize,
}

pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    total_pages: usize,
    start: usize,
    end: usize,
    page: PageInfo,
    byte: ByteInfo,

    // inner: BitAllocUsed,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    /// Creates a new empty `EarlyAllocator`.
    pub const fn new() -> Self {
        Self {
            total_pages: 0,
            start: 0,
            end: 0,
            page: PageInfo {
                pos: 0,
                used_pages: 0,
            },
            byte: ByteInfo {
                pos: 0,
                pos_byte: 0,
                used_bytes: 0,
                used_pages: 0,
            },
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        assert!(PAGE_SIZE.is_power_of_two());
        self.start = super::align_up(start, PAGE_SIZE);
        self.end = super::align_down(start + size, PAGE_SIZE);
        self.total_pages = (self.end - self.start) / PAGE_SIZE;
        self.byte.pos = self.start;
        self.byte.pos_byte = self.start;
        self.page.pos = self.end;
    }

    fn add_memory(&mut self, _start: usize, _size: usize) -> AllocResult {
        Err(AllocError::InvalidParam) // unsupported
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }
        let align_pow2 = align_pow2 / PAGE_SIZE;
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        let align_log2 = align_pow2.trailing_zeros() as usize;
        // self.page.pos -= PAGE_SIZE;
        // self.page.used_pages += 1;
        // Ok(self.page.pos)
        match num_pages.cmp(&0) {
            core::cmp::Ordering::Greater =>{
                let pos_alloc = ((self.page.pos - num_pages * PAGE_SIZE) >> align_log2) << align_log2;
                if self.byte.pos > pos_alloc {
                    return Err(AllocError::NoMemory);
                }
                self.page.pos = pos_alloc;
                self.page.used_pages = (self.end - self.page.pos) / PAGE_SIZE;
                return Ok(self.page.pos)
            },
            _ => return Err(AllocError::InvalidParam),
        }
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // unsupported
    }

    fn total_pages(&self) -> usize {
        self.total_pages
    }

    fn used_pages(&self) -> usize {
        self.page.used_pages
    }

    fn available_pages(&self) -> usize {
        self.total_pages - self.page.used_pages - self.byte.used_pages
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let pos_alloc = ((self.byte.pos_byte - 1) / layout.align() + 1) * layout.align();
        if pos_alloc + layout.size() > self.page.pos {
            return Err(AllocError::NoMemory);
        }
        self.byte.pos_byte = pos_alloc + layout.size();
        self.byte.used_bytes += layout.size();
        self.byte.pos = ((self.byte.pos_byte - 1) / PAGE_SIZE + 1) * PAGE_SIZE;
        self.byte.used_pages = (self.byte.pos - self.start) / PAGE_SIZE;
        let pos_byte_ptr = pos_alloc as *mut u8;
        if !pos_byte_ptr.is_null() {
            return Ok(NonNull::new(pos_byte_ptr).expect("Pointer is null"));
        } else {
            return Err(AllocError::NoMemory);
        }
    }

    fn dealloc(&mut self, _pos: NonNull<u8>, layout: Layout) {
        self.byte.used_bytes -= layout.size();
        if self.byte.used_bytes == 0 {
            self.byte = ByteInfo {
                pos: self.start,
                pos_byte: self.start,
                used_pages: 0,
                used_bytes: 0,
            }
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.byte.pos_byte - self.start
    }

    fn available_bytes(&self) -> usize {
        self.page.pos - self.byte.pos_byte
    }
}
