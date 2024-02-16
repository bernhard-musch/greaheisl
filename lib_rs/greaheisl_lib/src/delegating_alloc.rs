use core::alloc::GlobalAlloc;
use core::ffi::c_void;
use core::ptr::null_mut;

use round_mult::NonZeroPow2;

/// This function needs to be called before any allocation takes place.
/// The implementation is not thread safe. Make sure no threads performing
/// allocations have been spawned before this function has completed.
pub unsafe fn init_delegating_allocatator(
    aligned_alloc: Option<unsafe extern "C" fn(usize, usize) -> *mut c_void>,
    free: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    DELEGATING_ALLOCATOR = DelegatingAllocator {
        aligned_alloc,
        free,
    }
}

struct DelegatingAllocator {
    pub aligned_alloc: Option<unsafe extern "C" fn(usize, usize) -> *mut c_void>,
    pub free: Option<unsafe extern "C" fn(*mut c_void)>,
}

#[global_allocator]
static mut DELEGATING_ALLOCATOR: DelegatingAllocator = DelegatingAllocator {
    aligned_alloc: None,
    free: None,
};

unsafe impl Sync for DelegatingAllocator {}

unsafe impl GlobalAlloc for DelegatingAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if let Some(aligned_alloc) = self.aligned_alloc {
            let alignment = layout.align();
            let alignpow2 = NonZeroPow2::new(alignment).unwrap();
            let size = round_mult::up(layout.size(), alignpow2).unwrap();
            aligned_alloc(alignment, size) as *mut u8
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        if let Some(free) = self.free {
            free(ptr as *mut c_void);
        } else {
            panic!("Memory Free function not initialized!");
        }
    }
}
