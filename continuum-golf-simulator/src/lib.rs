// Continuum Golf Simulator - Rust Implementation
//
// This library provides a high-performance golf wagering simulator that models:
// - Proprietary odds engine with dynamic P_max calculations
// - Player skill adaptation using Kalman filtering
// - Venue economics and tournament simulations
//
// The simulator ensures fairness (equal EV across all handicaps) while maintaining
// target RTP (Return to Player) percentages: 86% (short), 88% (mid), 90% (long)

pub mod math;
pub mod models;
pub mod simulators;
pub mod analytics;
pub mod anti_cheat;
pub mod config;

// Re-export commonly used types
pub use math::{distributions, integration, kalman};
pub use models::{hole, player, shot};
pub use simulators::{player_session, venue, tournament};
pub use analytics::{metrics, export};
