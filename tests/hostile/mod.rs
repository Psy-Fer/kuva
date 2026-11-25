//! Shared setup for the hostile-input tests (`*_hostile.rs`).
//!
//! Including this module with `mod hostile;` installs a global allocator that
//! caps the test binary at 2 GB, and provides [`bounded`] to run a closure on a
//! small stack with a time limit. A regression that loops or recurses on bad
//! input then fails the test quickly instead of hanging or using all of the
//! machine's memory. Only include it from hostile-input test files: the cap
//! applies to every test in the binary.

#![allow(dead_code)]

use std::alloc::{GlobalAlloc, Layout as AllocLayout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use kuva::backend::svg::SvgBackend;
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

/// Allocator that refuses to hold more than 2 GB at once. When a walk loops
/// and keeps growing a buffer, an allocation fails and the process aborts with
/// "memory allocation failed".
struct Capped;

const ALLOC_CAP: usize = 2 << 30;
static IN_USE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Capped {
    unsafe fn alloc(&self, layout: AllocLayout) -> *mut u8 {
        if IN_USE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size() > ALLOC_CAP {
            IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
            return std::ptr::null_mut();
        }
        let p = unsafe { System.alloc(layout) };
        if p.is_null() {
            IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: AllocLayout) {
        unsafe { System.dealloc(ptr, layout) };
        IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: Capped = Capped;

/// Run `f` on a thread with a `stack`-byte stack and fail if it overflows,
/// panics or takes longer than 10 seconds.
///
/// On a timeout the whole test process exits. Panicking would only fail this
/// test and leave the stuck thread running (and possibly allocating) while the
/// other tests finish.
pub fn bounded<T: Send + 'static>(stack: usize, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .stack_size(stack)
        .spawn(move || {
            let _ = tx.send(f());
        })
        .unwrap();
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(v) => v,
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("did not finish (panic or stack overflow)")
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            eprintln!("did not finish within 10 seconds (hang); exiting the test process");
            std::process::exit(1);
        }
    }
}

/// Render one plot to an SVG string with an automatic layout.
pub fn render(plot: Plot) -> String {
    let plots = vec![plot];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}
