use std::alloc::{Layout, System};

use stats_alloc::StatsAlloc;

use crate::RetryAlloc;

static RETRY: RetryAlloc = RetryAlloc::new(System);
#[global_allocator]
static GLOBAL: StatsAlloc<&RetryAlloc> = StatsAlloc::new(&RETRY);

#[test]
fn test_fail() {
    let p = unsafe { std::alloc::alloc(Layout::array::<u32>(isize::MAX as usize / 4).unwrap()) };
    assert!(p.is_null());
    assert!(RETRY.number_of_retries() > 0);
}
