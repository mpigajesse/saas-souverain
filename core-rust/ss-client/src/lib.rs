pub mod invariants;
pub mod license;
pub mod offline;
pub mod pairing;

// Auto-generated gRPC Protobuf bindings (Mission B)
pub mod pb {
    tonic::include_proto!("amane.framework.v1");
}

pub use invariants::{InvariantChecker, InvariantError, InvoiceRecord, StockMovement};
pub use license::{LicenseError, LicensePayload, LicenseValidator, SignedLicenseToken};
pub use offline::{OfflineCacheManager, OfflineError, OperationMode};
pub use pairing::{PairingError, PairingInvitationToken, PairingManager, SealedAkPayload};
pub use pb::amane_service_client::AmaneServiceClient;
