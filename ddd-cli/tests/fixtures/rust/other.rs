//! A second module, so `lib.rs` can hold an impl whose trait and self type
//! are both declared elsewhere — the shape whose coherence the adapter
//! cannot settle from one document.

pub trait Remote {
    fn ping(&self) -> u32;
}

pub struct Far;
