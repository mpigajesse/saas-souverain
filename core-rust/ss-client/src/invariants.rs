use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvariantError {
    #[error("Stock insuffisant : stock actuel ({current}), quantité demandée ({requested})")]
    InsufficientStock { current: i64, requested: i64 },

    #[error("Rupture de numérotation séquentielle : attendu ({expected}), reçu ({received})")]
    SequenceBreak { expected: u64, received: u64 },

    #[error("Montant invalide : {0}")]
    InvalidAmount(String),
}

/// Représente un mouvement de stock à valider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovement {
    pub item_id: String,
    pub current_quantity: i64,
    pub change_quantity: i64, // Positif = Entrée, Négatif = Vente/Sortie
}

/// Représente une facture à valider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceRecord {
    pub invoice_number: u64,
    pub expected_next_number: u64,
    pub total_amount: f64,
}

pub struct InvariantChecker;

impl InvariantChecker {
    /// Règle 1 : Interdiction stricte des stocks négatifs
    pub fn validate_stock(movement: &StockMovement) -> Result<i64, InvariantError> {
        let new_quantity = movement.current_quantity + movement.change_quantity;
        if new_quantity < 0 {
            return Err(InvariantError::InsufficientStock {
                current: movement.current_quantity,
                requested: movement.change_quantity.abs(),
            });
        }
        Ok(new_quantity)
    }

    /// Règle 2 : Numérotation séquentielle stricte et sans trou des factures
    pub fn validate_invoice_sequence(invoice: &InvoiceRecord) -> Result<(), InvariantError> {
        if invoice.invoice_number != invoice.expected_next_number {
            return Err(InvariantError::SequenceBreak {
                expected: invoice.expected_next_number,
                received: invoice.invoice_number,
            });
        }
        if invoice.total_amount < 0.0 {
            return Err(InvariantError::InvalidAmount(
                "Le montant total de la facture ne peut pas être négatif".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_stock_reduction() {
        let movement = StockMovement {
            item_id: "ITEM-001".into(),
            current_quantity: 10,
            change_quantity: -3,
        };
        let result = InvariantChecker::validate_stock(&movement);
        assert_eq!(result.unwrap(), 7);
    }

    #[test]
    fn test_invalid_negative_stock() {
        let movement = StockMovement {
            item_id: "ITEM-001".into(),
            current_quantity: 2,
            change_quantity: -5,
        };
        let result = InvariantChecker::validate_stock(&movement);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_sequence() {
        let inv = InvoiceRecord {
            invoice_number: 104,
            expected_next_number: 104,
            total_amount: 150.0,
        };
        assert!(InvariantChecker::validate_invoice_sequence(&inv).is_ok());
    }
}
