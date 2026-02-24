pub mod export;
/// Analytics module for metrics calculation and data export
///
/// This module provides:
/// - Expected value calculations and validation
/// - RTP verification across skill levels
/// - Fairness metrics (EV equality)
/// - Kalman filter convergence analysis
/// - Data export utilities (CSV, JSON)
pub mod metrics;

pub use export::*;
pub use metrics::*;
