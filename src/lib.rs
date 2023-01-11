//! A session library
#![cfg_attr(nightly, feature(doc_cfg))]
#![warn(missing_docs)]

pub use self::{
    collector::{SessionCollector, SessionCollectorHandle},
    manager::SessionManager,
    session::{Session, SessionError},
};

mod collector;
mod manager;
mod session;
mod utils;
mod value;

/// Store backend implementations
pub mod backend;
