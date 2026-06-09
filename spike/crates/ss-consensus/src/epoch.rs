use serde::{Deserialize, Serialize};

/// Jeton d'époque : compteur monotone incrémenté à chaque élection de nœud actif.
/// Un nœud qui revient en ligne après une coupure ne peut pas reprendre la main
/// si son époque est inférieure à l'époque courante du cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EpochToken(pub u64);

impl EpochToken {
    pub fn initial() -> Self {
        Self(1)
    }

    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn is_fresher_than(&self, other: &Self) -> bool {
        self.0 > other.0
    }
}

impl std::fmt::Display for EpochToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "epoch({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_is_one() {
        assert_eq!(EpochToken::initial().value(), 1);
    }

    #[test]
    fn increment_is_monotone() {
        let e = EpochToken::initial();
        let e2 = e.increment();
        let e3 = e2.increment();
        assert_eq!(e2.value(), 2);
        assert_eq!(e3.value(), 3);
        assert!(e3.is_fresher_than(&e2));
        assert!(!e2.is_fresher_than(&e3));
    }

    #[test]
    fn display_format() {
        let e = EpochToken(42);
        assert_eq!(e.to_string(), "epoch(42)");
    }
}
