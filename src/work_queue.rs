use crossbeam_channel::{bounded, Receiver, Sender};
use rug::Integer;

/// A batch of sieve survivors to be Fermat-tested.
pub struct CandidateBatch {
    /// The base candidate for this sieve iteration:
    /// `candidate = primorial * f + first_candidate` for each survivor `f`.
    pub first_candidate: Integer,
    /// Sieve-surviving factor indices within one sieve iteration.
    pub survivors: Vec<u32>,
}

/// Bounded MPMC work queue connecting sieve workers to test workers.
pub struct WorkQueue {
    pub sender: Sender<CandidateBatch>,
    pub receiver: Receiver<CandidateBatch>,
}

impl WorkQueue {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self { sender, receiver }
    }
}
