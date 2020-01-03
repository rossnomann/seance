//! A session library
#![warn(missing_docs)]

mod collector;
mod manager;
mod session;
mod utils;
mod value;

pub use self::{
    collector::{SessionCollector, SessionCollectorHandle},
    manager::SessionManager,
    session::{Session, SessionError},
};

/// Store backend implementations
pub mod backend;
