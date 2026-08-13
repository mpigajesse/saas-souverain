pub mod invariants;
pub mod offline;

// Auto-generated gRPC Protobuf bindings (Mission B)
pub mod pb {
    tonic::include_proto!("amane.framework.v1");
}

pub use invariants::{InvariantChecker, InvariantError, InvoiceRecord, StockMovement};
pub use offline::{OfflineCacheManager, OfflineError, OperationMode};
pub use pb::amane_service_client::AmaneServiceClient;
