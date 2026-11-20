//! A recording transport for the egress port: it answers from a queue and
//! keeps every request it was given, so a test can assert what would have left
//! the host without any network.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};

use usher_core::{Response, Transport, UsherError, WireRequest};

#[derive(Default)]
struct State {
    answers: VecDeque<Result<Response, UsherError>>,
    sent: Vec<WireRequest>,
}

/// Answers each request with the next queued answer and records the request.
/// Clones share one queue and one record, so a test keeps a clone to inspect
/// after the port took the other.
#[derive(Clone, Default)]
pub struct RecordingTransport {
    state: Arc<Mutex<State>>,
}

impl RecordingTransport {
    /// A transport with nothing queued. A request with no answer queued fails.
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Queues one answer.
    #[must_use]
    pub fn answering(self, answer: Result<Response, UsherError>) -> Self {
        self.lock().answers.push_back(answer);
        self
    }

    /// Queues a 200 answer with this body.
    #[must_use]
    pub fn answering_ok(self, body: impl Into<Vec<u8>>) -> Self {
        self.answering(Ok(Response {
            status: 200,
            location: None,
            body: body.into(),
        }))
    }

    /// Queues a redirect answer to this location.
    #[must_use]
    pub fn answering_redirect(self, location: impl Into<String>) -> Self {
        self.answering(Ok(Response {
            status: 302,
            location: Some(location.into()),
            body: Vec::new(),
        }))
    }

    /// Every request received so far, in order.
    pub fn sent(&self) -> Vec<WireRequest> {
        self.lock().sent.clone()
    }
}

impl Transport for RecordingTransport {
    fn send(&self, request: &WireRequest) -> Result<Response, UsherError> {
        let mut state = self.lock();
        state.sent.push(request.clone());
        state.answers.pop_front().unwrap_or_else(|| {
            Err(UsherError::new(
                "recording transport",
                "an answer is queued",
                "no answer queued",
                "queue an answer before the call",
            ))
        })
    }
}
