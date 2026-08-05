//! Versioned transport messages shared by the server and UI.

pub mod client;
pub mod server;
pub mod snapshot;
pub mod version;

pub use client::{ClientEnvelope, ClientMessage};
pub use version::PROTOCOL_VERSION;
