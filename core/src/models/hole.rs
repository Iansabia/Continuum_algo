// Hole configuration and payout calculations
//
// Defines the 8 hole configurations with their unique parameters:
// - Distance (yds)
// - Scoring radius (d_max in feet)
// - Return to Player (RTP: 0.86-0.90)
// - Steepness factor (k: 5.0-6.5)

use serde::{Deserialize, Serialize};

/// Club category based on distance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClubCategory {
    /// Short shots: 75-125 yards (wedges)
    Wedge,
    /// Medium shots: 150-175 yards (mid irons)
    MidIron,
    /// Long shots: 200-250 yards (long irons)
    LongIron,
}

impl ClubCategory {
    /// Get category from distance
    pub fn from_distance(distance_yds: u16) -> Self {
        match distance_yds {
            0..=130 => ClubCategory::Wedge,
            131..=185 => ClubCategory::MidIron,
            _ => ClubCategory::LongIron,
        }
    }
}

/// Hole configuration with scoring parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hole {
    /// Hole number (1-8)
    pub id: u8,
    /// Distance in yards
    pub distance_yds: u16,
    /// Maximum scoring radius in feet
    pub d_max_ft: f64,
    /// Return to player (0.86-0.90)
    pub rtp: f64,
    /// Steepness factor for payout curve (5.0-6.5)
    pub k: f64,
    /// Club category
    pub category: ClubCategory,
}

impl Hole {
    /// Create a new hole configuration
    pub fn new(
        id: u8,
        distance_yds: u16,
        d_max_ft: f64,
        rtp: f64,
        k: f64,
    ) -> Self {
        let category = ClubCategory::from_distance(distance_yds);
        Hole {
            id,
            distance_yds,
            d_max_ft,
            rtp,
            k,
            category,
        }
    }

    /// Calculate payout multiplier for a given miss distance
    ///
    /// # Formula
    /// If d ≤ d_max: P(d) = P_max * (1 - d/d_max)^k
    /// If d > d_max: P(d) = 0
    ///
    /// # Arguments
    /// * `miss_distance` - Miss distance in feet
    /// * `p_max` - Maximum payout multiplier (calculated from player skill)
    ///
    /// # Returns
    /// Payout multiplier (e.g., 5.0 = 5× return on wager)
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::models::hole::Hole;
    ///
    /// let hole = Hole::new(1, 75, 17.95, 0.86, 5.0);
    /// let payout = hole.calculate_payout(10.0, 12.0);
    /// assert!(payout > 0.0);
    /// assert!(payout <= 12.0);
    /// ```
    pub fn calculate_payout(&self, miss_distance: f64, p_max: f64) -> f64 {
        if miss_distance > self.d_max_ft {
            return 0.0;
        }

        // P(d) = P_max * (1 - d/d_max)^k
        let normalized = 1.0 - (miss_distance / self.d_max_ft);
        p_max * normalized.powf(self.k)
    }

    /// Calculate breakeven radius for a given P_max
    ///
    /// The breakeven radius is the distance at which the payout equals
    /// the wager (multiplier = 1.0).
    ///
    /// # Formula
    /// d_break = d_max * (1 - P_max^(-1/k))
    ///
    /// # Arguments
    /// * `p_max` - Maximum payout multiplier
    ///
    /// # Returns
    /// Breakeven radius in feet
    ///
    /// # Example
    /// ```
    /// use continuum_golf_simulator::models::hole::Hole;
    ///
    /// let hole = Hole::new(4, 150, 47.58, 0.88, 6.0);
    /// let breakeven = hole.calculate_breakeven_radius(10.0);
    /// // At this distance, payout should be 1.0×
    /// let payout = hole.calculate_payout(breakeven, 10.0);
    /// assert!((payout - 1.0).abs() < 0.01);
    /// ```
    pub fn calculate_breakeven_radius(&self, p_max: f64) -> f64 {
        if p_max <= 1.0 {
            return 0.0; // No breakeven possible
        }

        // d_break = d_max * (1 - P_max^(-1/k))
        self.d_max_ft * (1.0 - p_max.powf(-1.0 / self.k))
    }

    /// Get expected multiplier at center (d=0)
    pub fn max_payout(&self, p_max: f64) -> f64 {
        p_max
    }

    /// Get the club category for this hole
    pub fn get_category(&self) -> ClubCategory {
        self.category
    }
}

/// The 8 official hole configurations from the business plan
///
/// Configuration format (adjusted for 15% target hold):
/// - All holes: RTP 85.0%, targeting 15% hold
/// - Different k values provide varying payout curves
pub const HOLE_CONFIGURATIONS: [Hole; 8] = [
    // Short holes (Wedge category)
    Hole {
        id: 1,
        distance_yds: 75,
        d_max_ft: 17.95,
        rtp: 0.85,
        k: 5.0,
        category: ClubCategory::Wedge,
    },
    Hole {
        id: 2,
        distance_yds: 100,
        d_max_ft: 25.69,
        rtp: 0.85,
        k: 5.0,
        category: ClubCategory::Wedge,
    },
    Hole {
        id: 3,
        distance_yds: 125,
        d_max_ft: 36.71,
        rtp: 0.85,
        k: 5.5,
        category: ClubCategory::Wedge,
    },
    // Mid holes (MidIron category)
    Hole {
        id: 4,
        distance_yds: 150,
        d_max_ft: 47.58,
        rtp: 0.85,
        k: 6.0,
        category: ClubCategory::MidIron,
    },
    Hole {
        id: 5,
        distance_yds: 175,
        d_max_ft: 59.09,
        rtp: 0.85,
        k: 6.0,
        category: ClubCategory::MidIron,
    },
    // Long holes (LongIron category)
    Hole {
        id: 6,
        distance_yds: 200,
        d_max_ft: 73.58,
        rtp: 0.85,
        k: 6.5,
        category: ClubCategory::LongIron,
    },
    Hole {
        id: 7,
        distance_yds: 225,
        d_max_ft: 84.84,
        rtp: 0.85,
        k: 6.5,
        category: ClubCategory::LongIron,
    },
    Hole {
        id: 8,
        distance_yds: 250,
        d_max_ft: 101.14,
        rtp: 0.85,
        k: 6.5,
        category: ClubCategory::LongIron,
    },
];

/// Get hole by ID (1-8)
pub fn get_hole_by_id(id: u8) -> Option<&'static Hole> {
    if id < 1 || id > 8 {
        return None;
    }
    Some(&HOLE_CONFIGURATIONS[(id - 1) as usize])
}

/// Get all holes for a specific category
pub fn get_holes_by_category(category: ClubCategory) -> Vec<&'static Hole> {
    HOLE_CONFIGURATIONS
        .iter()
        .filter(|h| h.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_club_category_from_distance() {
        assert_eq!(ClubCategory::from_distance(75), ClubCategory::Wedge);
        assert_eq!(ClubCategory::from_distance(125), ClubCategory::Wedge);
        assert_eq!(ClubCategory::from_distance(150), ClubCategory::MidIron);
        assert_eq!(ClubCategory::from_distance(175), ClubCategory::MidIron);
        assert_eq!(ClubCategory::from_distance(200), ClubCategory::LongIron);
        assert_eq!(ClubCategory::from_distance(250), ClubCategory::LongIron);
    }

    #[test]
    fn test_hole_configurations_exist() {
        assert_eq!(HOLE_CONFIGURATIONS.len(), 8);

        // Check first hole
        let h1 = &HOLE_CONFIGURATIONS[0];
        assert_eq!(h1.id, 1);
        assert_eq!(h1.distance_yds, 75);
        assert_eq!(h1.rtp, 0.85);

        // Check last hole
        let h8 = &HOLE_CONFIGURATIONS[7];
        assert_eq!(h8.id, 8);
        assert_eq!(h8.distance_yds, 250);
        assert_eq!(h8.rtp, 0.85);
    }

    #[test]
    fn test_payout_at_center() {
        let hole = Hole::new(1, 75, 17.95, 0.86, 5.0);
        let p_max = 10.0;

        // At center (d=0), payout should equal P_max
        let payout = hole.calculate_payout(0.0, p_max);
        assert_eq!(payout, p_max);
    }

    #[test]
    fn test_payout_at_edge() {
        let hole = Hole::new(1, 75, 17.95, 0.86, 5.0);
        let p_max = 10.0;

        // At d_max, payout should be 0
        let payout = hole.calculate_payout(17.95, p_max);
        assert_relative_eq!(payout, 0.0, epsilon = 0.01);
    }

    #[test]
    fn test_payout_beyond_edge() {
        let hole = Hole::new(1, 75, 17.95, 0.86, 5.0);
        let p_max = 10.0;

        // Beyond d_max, payout is 0
        let payout = hole.calculate_payout(20.0, p_max);
        assert_eq!(payout, 0.0);
    }

    #[test]
    fn test_payout_monotonic() {
        let hole = Hole::new(4, 150, 47.58, 0.88, 6.0);
        let p_max = 10.0;

        // Payout should decrease as distance increases
        let payout_5 = hole.calculate_payout(5.0, p_max);
        let payout_10 = hole.calculate_payout(10.0, p_max);
        let payout_20 = hole.calculate_payout(20.0, p_max);

        assert!(payout_5 > payout_10);
        assert!(payout_10 > payout_20);
    }

    #[test]
    fn test_breakeven_radius() {
        let hole = Hole::new(4, 150, 47.58, 0.88, 6.0);
        let p_max = 10.0;

        let breakeven = hole.calculate_breakeven_radius(p_max);

        // At breakeven distance, payout should be 1.0×
        let payout = hole.calculate_payout(breakeven, p_max);
        assert_relative_eq!(payout, 1.0, epsilon = 0.01);
    }

    #[test]
    fn test_breakeven_formula() {
        // Test the breakeven formula: d_break = d_max * (1 - P_max^(-1/k))
        let hole = Hole::new(6, 200, 73.58, 0.90, 6.5);
        let p_max = 12.0;

        let breakeven = hole.calculate_breakeven_radius(p_max);
        let expected = 73.58 * (1.0 - 12.0_f64.powf(-1.0 / 6.5));

        assert_relative_eq!(breakeven, expected, epsilon = 0.01);
    }

    #[test]
    fn test_get_hole_by_id() {
        let hole1 = get_hole_by_id(1).unwrap();
        assert_eq!(hole1.id, 1);
        assert_eq!(hole1.distance_yds, 75);

        let hole8 = get_hole_by_id(8).unwrap();
        assert_eq!(hole8.id, 8);
        assert_eq!(hole8.distance_yds, 250);

        // Invalid IDs
        assert!(get_hole_by_id(0).is_none());
        assert!(get_hole_by_id(9).is_none());
    }

    #[test]
    fn test_get_holes_by_category() {
        let wedges = get_holes_by_category(ClubCategory::Wedge);
        assert_eq!(wedges.len(), 3); // H1, H2, H3

        let mid_irons = get_holes_by_category(ClubCategory::MidIron);
        assert_eq!(mid_irons.len(), 2); // H4, H5

        let long_irons = get_holes_by_category(ClubCategory::LongIron);
        assert_eq!(long_irons.len(), 3); // H6, H7, H8
    }

    #[test]
    fn test_rtp_progression() {
        // All holes should have RTP of 0.85
        let h1 = get_hole_by_id(1).unwrap();
        assert_eq!(h1.rtp, 0.85);

        let h4 = get_hole_by_id(4).unwrap();
        assert_eq!(h4.rtp, 0.85);

        let h8 = get_hole_by_id(8).unwrap();
        assert_eq!(h8.rtp, 0.85);
    }
}
