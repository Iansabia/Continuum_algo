/// Analytics module for metrics calculation and data export
///
/// This module provides:
/// - Expected value calculations and validation
/// - RTP verification across skill levels
/// - Fairness metrics (EV equality)
/// - Kalman filter convergence analysis
/// - Data export utilities (CSV, JSON)

pub mod metrics;
pub mod export;

pub use metrics::*;
pub use export::*;
