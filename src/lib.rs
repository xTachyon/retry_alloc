use std::{
    alloc::{GlobalAlloc, Layout, System},
    ptr::null_mut,
    sync::atomic::{AtomicU64, Ordering},
    thread::sleep,
    time::Duration,
};

#[non_exhaustive]
pub struct RetryConfig {
    pub time_to_wait: Duration,
    pub max_retries: u32,
}

impl RetryConfig {
    pub const fn new_v1(time_to_wait: Duration, max_retries: u32) -> RetryConfig {
        RetryConfig {
            time_to_wait,
            max_retries,
        }
    }
}

pub struct RetryAlloc<T: GlobalAlloc = System> {
    inner: T,
    config: RetryConfig,
    number_of_retries: AtomicU64,
}

impl<T: GlobalAlloc> RetryAlloc<T> {
    #[inline]
    pub const fn with_config(alloc: T, config: RetryConfig) -> Self {
        Self {
            inner: alloc,
            config,
            number_of_retries: AtomicU64::new(0),
        }
    }

    pub const fn new(alloc: T) -> Self {
        Self::with_config(
            alloc,
            RetryConfig {
                time_to_wait: Duration::from_millis(50),
                max_retries: 10,
            },
        )
    }

    #[inline]
    pub fn inner(&self) -> &T {
        &self.inner
    }

    #[inline]
    pub fn number_of_retries(&self) -> u64 {
        self.number_of_retries.load(Ordering::Relaxed)
    }

    #[cold]
    unsafe fn alloc_slow(&self, layout: Layout) -> *mut u8 {
        for _ in 0..self.config.max_retries {
            sleep(self.config.time_to_wait);
            self.number_of_retries.fetch_add(1, Ordering::Relaxed);

            let r = self.inner.alloc(layout);
            if !r.is_null() {
                return r;
            }
        }

        null_mut()
    }

    #[cold]
    unsafe fn realloc_slow(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        for _ in 0..self.config.max_retries {
            sleep(self.config.time_to_wait);
            self.number_of_retries.fetch_add(1, Ordering::Relaxed);

            let r = self.inner.realloc(ptr, layout, new_size);
            if !r.is_null() {
                return r;
            }
        }

        null_mut()
    }
}

unsafe impl<T: GlobalAlloc> GlobalAlloc for RetryAlloc<T> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let r = self.inner.alloc(layout);
        if r.is_null() {
            self.alloc_slow(layout)
        } else {
            r
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.inner.alloc_zeroed(layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let r = self.inner.realloc(ptr, layout, new_size);
        if r.is_null() {
            self.realloc_slow(ptr, layout, new_size)
        } else {
            r
        }
    }
}
