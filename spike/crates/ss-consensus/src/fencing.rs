use crate::EpochToken;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FencingResult {
    /// L'époque présentée est à jour : l'opération peut continuer.
    Allowed,
    /// L'époque présentée est obsolète : le nœud doit s'isoler.
    Fenced {
        claimed: EpochToken,
        current: EpochToken,
    },
}

/// Vérifie si un nœud est autorisé à écrire.
/// `claimed_epoch` : époque que le nœud pense avoir.
/// `current_epoch` : époque du cluster selon la supervision.
pub fn check_fencing(claimed_epoch: EpochToken, current_epoch: EpochToken) -> FencingResult {
    if claimed_epoch >= current_epoch {
        FencingResult::Allowed
    } else {
        FencingResult::Fenced {
            claimed: claimed_epoch,
            current: current_epoch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_epoch_allowed() {
        let result = check_fencing(EpochToken(5), EpochToken(5));
        assert_eq!(result, FencingResult::Allowed);
    }

    #[test]
    fn stale_epoch_fenced() {
        let result = check_fencing(EpochToken(3), EpochToken(7));
        assert!(matches!(result, FencingResult::Fenced { .. }));
    }

    #[test]
    fn fresh_epoch_allowed() {
        // Un nœud qui a une époque > celle connue (cas improbable mais toléré)
        let result = check_fencing(EpochToken(8), EpochToken(5));
        assert_eq!(result, FencingResult::Allowed);
    }

    #[test]
    fn fenced_contains_both_epochs() {
        let result = check_fencing(EpochToken(2), EpochToken(9));
        match result {
            FencingResult::Fenced { claimed, current } => {
                assert_eq!(claimed.value(), 2);
                assert_eq!(current.value(), 9);
            }
            FencingResult::Allowed => panic!("expected Fenced"),
        }
    }
}
