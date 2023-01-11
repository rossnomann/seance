use std::time::Duration;

use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::interval,
};

use crate::{backend::SessionBackend, utils::now};

/// Garbage collector for sessions
pub struct SessionCollector<B> {
    backend: B,
    period: Duration,
    lifetime: Duration,
    sender: Sender<()>,
    receiver: Receiver<()>,
}

impl<B> SessionCollector<B>
where
    B: SessionBackend,
{
    /// Creates a new collector
    ///
    /// GC allows to remove all sessions with age longer than given `lifetime`
    ///
    /// # Arguments
    ///
    /// * backend - Store backend
    /// * period - Interval between GC calls
    /// * lifetime - Minimum session lifetime
    pub fn new(backend: B, period: Duration, lifetime: Duration) -> Self {
        let (sender, receiver) = channel(1);
        Self {
            backend,
            period,
            lifetime,
            sender,
            receiver,
        }
    }

    /// Returns a session collector handle
    pub fn get_handle(&self) -> SessionCollectorHandle {
        SessionCollectorHandle {
            sender: self.sender.clone(),
        }
    }

    async fn collect(&mut self) -> Result<(), String> {
        let lifetime = self.lifetime.as_secs();
        let session_ids = self.backend.get_sessions().await.map_err(|err| err.to_string())?;
        let timestamp = now().map_err(|err| err.to_string())?;
        for session_id in session_ids {
            if let Some(age) = self
                .backend
                .get_session_age(&session_id)
                .await
                .map_err(|err| err.to_string())?
            {
                if timestamp - age >= lifetime {
                    self.backend
                        .remove_session(&session_id)
                        .await
                        .map_err(|err| err.to_string())?;
                }
            }
        }
        Ok(())
    }

    /// Starts GC loop
    pub async fn run(&mut self) {
        let mut interval = interval(self.period);
        loop {
            if self.receiver.try_recv().is_ok() {
                self.receiver.close();
                break;
            }
            interval.tick().await;
            if let Err(err) = self.collect().await {
                log::error!("An error occurred in session GC: {}", err)
            }
        }
    }
}

/// GC handle
pub struct SessionCollectorHandle {
    sender: Sender<()>,
}

impl SessionCollectorHandle {
    /// Stop GC loop
    pub async fn shutdown(self) {
        let _ = self.sender.send(()).await;
    }
}
