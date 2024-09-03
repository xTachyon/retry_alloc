A global allocator wrapper that will retry allocations a few times if one allocation fails.

The assumption is that when one allocation fails, it is because the system is out of memory. In that case, the following things might happen:
1. The system kills the offending process that uses too much memory;
2. The system frees some disk or any other caches that it holds;
3. Other processes die because they're out of memory.

In all the cases, some memory gets freed on the system. This means that retrying an allocation after waiting some time might succeed, at the expense of a latency spike in the application, which is usually a better outcome than the application dying.

An allocation failure is still possible, but this makes it more unlikely.

Example
```rs
#[global_allocator]
static GLOBAL: RetryAlloc = RetryAlloc::new(System);
```

With `stats_alloc`:
```rs
static RETRY: RetryAlloc = RetryAlloc::new(System);
#[global_allocator]
static GLOBAL: StatsAlloc<&RetryAlloc> = StatsAlloc::new(&RETRY);
```
In this case, `RETRY` has to be a separate static because `StatsAlloc` doesn't have a way to access the inner alloc in order to retrieve fail statistics (if you need that), and having it in the inverse order might result in some stats being counted wrong due to `StatsAlloc` still counting if the allocation failed.
