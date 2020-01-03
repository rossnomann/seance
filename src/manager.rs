use crate::{backend::SessionBackend, session::Session};
use std::sync::Arc;
use tokio::sync::Mutex;

/// A session manager
#[derive(Clone)]
pub struct SessionManager<B> {
    backend: Arc<Mutex<B>>,
}

impl<B> SessionManager<B>
where
    B: SessionBackend,
{
    /// Creates a new session manager
    ///
    /// # Arguments
    ///
    /// * backend - A session backend
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(Mutex::new(backend)),
        }
    }

    /// Returns a session for ID
    pub fn get_session<I>(&self, id: I) -> Session<B>
    where
        I: Into<String>,
    {
        Session::new(id, self.backend.clone())
    }
}
