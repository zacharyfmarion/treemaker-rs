//! Numeric tolerances ported from Oriedita's `origami.Epsilon`.

/// Oriedita epsilon constants.
pub struct Epsilon;

impl Epsilon {
    /// Global scale used by Oriedita's current epsilon table.
    pub const FACTOR: f64 = 0.01;

    pub const UNKNOWN_05: f64 = Self::FACTOR * 0.5;
    pub const UNKNOWN_001: f64 = Self::FACTOR * 1E-2;
    pub const UNKNOWN_0001: f64 = Self::FACTOR * 1E-3;
    pub const UNKNOWN_1EN4: f64 = Self::FACTOR * 1E-4;
    pub const UNKNOWN_1EN5: f64 = Self::FACTOR * 1E-5;
    pub const UNKNOWN_1EN6: f64 = Self::FACTOR * 1E-6;
    pub const UNKNOWN_1EN7: f64 = Self::FACTOR * 1E-7;

    pub const PARALLEL_FOR_EDIT: f64 = Self::FACTOR * 0.1;
    pub const PARALLEL_FOR_FIX: f64 = Self::FACTOR * 0.5;
    pub const FLAT: f64 = Self::FACTOR * 1E-4;
    pub const QUAD_TREE_ITEM: f64 = Self::FACTOR * 0.5;
    pub const GRID_ANGLE_THRESHOLD: f64 = 1.0;
    pub const SWEET_DISTANCE: f64 = Self::FACTOR * 1E-10;
    pub const POINT: f64 = Self::FACTOR * 0.025;

    const ZERO_COMPARISON: f64 = Self::FACTOR * 1E-8;

    /// Oriedita's default "high" epsilon instance.
    pub const HIGH: HighEpsilon = HighEpsilon::new(Self::ZERO_COMPARISON);
}

/// Instance epsilon behavior matching Oriedita's `Epsilon.high`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighEpsilon {
    precision: f64,
}

impl HighEpsilon {
    pub const fn new(precision: f64) -> Self {
        Self { precision }
    }

    pub const fn precision(self) -> f64 {
        self.precision
    }

    pub fn eq0(self, value: f64) -> bool {
        -self.precision < value && value < self.precision
    }

    pub fn le0(self, value: f64) -> bool {
        value < self.precision
    }

    pub fn gt0(self, value: f64) -> bool {
        value > self.precision
    }
}

#[cfg(test)]
mod tests {
    use super::{Epsilon, HighEpsilon};

    #[test]
    fn constants_match_oriedita_factor_table() {
        assert_eq!(Epsilon::UNKNOWN_001, 0.0001);
        assert_eq!(Epsilon::PARALLEL_FOR_EDIT, 0.001);
        assert_eq!(Epsilon::POINT, 0.00025);
        assert_eq!(Epsilon::SWEET_DISTANCE, 0.000000000001);
    }

    #[test]
    fn high_epsilon_uses_strict_window() {
        let high = HighEpsilon::new(0.25);
        assert!(high.eq0(0.0));
        assert!(high.eq0(0.249));
        assert!(!high.eq0(0.25));
        assert!(high.le0(0.249));
        assert!(!high.le0(0.25));
        assert!(high.gt0(0.251));
    }
}
