// Continuum Golf Simulator - Rust Implementation
//
// This library provides a high-performance golf wagering simulator that models:
// - Proprietary odds engine with dynamic P_max calculations
// - Player skill adaptation using MCMC Bayesian inference
// - Venue economics and tournament simulations
//
// The simulator ensures fairness (equal EV across all handicaps) while maintaining
// target RTP (Return to Player) percentages: 86% (short), 88% (mid), 90% (long)

pub mod analytics;
pub mod anti_cheat;
pub mod config;
pub mod math;
pub mod models;
pub mod simulators;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use analytics::{export, metrics};
pub use math::{distributions, integration};
pub use models::{hole, player, shot};
pub use simulators::{player_session, tournament, venue};
