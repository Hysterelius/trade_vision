//! # trade_vision: an unofficial Rust API for TradingView
//! trade_vision is a pure Rust, library for retrieving stock data from TradingView.
//!
//! ## Features
//! - Realtime data
//!
//! ## Acknowledgements
//! This library is a rewrite and reinterpretation of Mathieu's excellent [JS TradingView
//! API library](https://github.com/Mathieu2301/TradingView-API)

mod error;
pub mod misc_requests;
pub mod protocol;
pub mod utils;

/// Contains modules for handling the events from TradingView. It manages
/// the setting up of the handlers and starting the session
mod quote;

pub use error::Error;
pub use quote::session;

mod chart;
// pub use chart::session;
