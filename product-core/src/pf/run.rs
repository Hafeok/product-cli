//! Bounded parallel execution of work units (§5 — the parallelism unit).
//!
//! Work units are independent by construction (frozen input, one artifact), so
//! they are embarrassingly parallel. `run_parallel` maps a work function over
//! items with at most `jobs` running at once, preserving input order in the
//! results. The §6.1 coherence bar is what makes the split safe; that gate runs
//! after the fan-out, in the caller.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Run `f` over `items` with at most `jobs` concurrent invocations. Results are
/// returned in input order. `f` receives the index + a reference to each item.
pub fn run_parallel<T, R, F>(items: Vec<T>, jobs: usize, f: F) -> Vec<R>
where
    T: Send + Sync,
    R: Send,
    F: Fn(usize, &T) -> R + Sync,
{
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let workers = jobs.max(1).min(n);
    let cursor = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, R)>> = Mutex::new(Vec::with_capacity(n));

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let i = cursor.fetch_add(1, Ordering::Relaxed);
                if i >= n {
                    break;
                }
                let r = f(i, &items[i]);
                results.lock().expect("results lock").push((i, r));
            });
        }
    });

    let mut collected = results.into_inner().expect("results into_inner");
    collected.sort_by_key(|(i, _)| *i);
    collected.into_iter().map(|(_, r)| r).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn maps_in_input_order() {
        let out = run_parallel(vec![1, 2, 3, 4, 5], 3, |_, x| x * 2);
        assert_eq!(out, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn runs_every_item_once() {
        let counter = AtomicUsize::new(0);
        let out = run_parallel(vec![(); 50], 8, |_, _| {
            counter.fetch_add(1, Ordering::Relaxed)
        });
        assert_eq!(counter.load(Ordering::Relaxed), 50);
        assert_eq!(out.len(), 50);
        // every index 0..50 was produced exactly once (order-preserved positions)
        let mut seen = out.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 50);
    }

    #[test]
    fn empty_input_is_empty_output() {
        let out: Vec<i32> = run_parallel(Vec::<i32>::new(), 4, |_, x| *x);
        assert!(out.is_empty());
    }

    #[test]
    fn jobs_clamped_below_item_count() {
        // jobs far above n must still run correctly
        let out = run_parallel(vec![10, 20], 99, |_, x| x + 1);
        assert_eq!(out, vec![11, 21]);
    }
}
