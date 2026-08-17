use ss_client::pb::amane_service_server::{AmaneService, AmaneServiceServer};
use ss_client::pb::{
    ReadRequest, ReadResponse, RegisterRequest, RegisterResponse, StatusRequest, StatusResponse,
    WriteRequest, WriteResponse,
};
use tonic::{transport::Server, Request, Response, Status};

/// Mock Server gRPC simulant le service d'orchestration Go (Mission C) pour les tests autonomes de la Mission B
#[derive(Default)]
pub struct MockAmaneServer;

#[tonic::async_trait]
impl AmaneService for MockAmaneServer {
    async fn register_node(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        
        if req.installation_id.is_empty() {
            return Ok(Response::new(RegisterResponse {
                accepted: false,
                current_epoch: 1,
                cluster_id: "".into(),
                error_message: "Installation ID manquant".into(),
            }));
        }

        Ok(Response::new(RegisterResponse {
            accepted: true,
            current_epoch: 1,
            cluster_id: "MOCK-CLUSTER-001".into(),
            error_message: "".into(),
        }))
    }

    async fn write_operation(
        &self,
        request: Request<WriteRequest>,
    ) -> Result<Response<WriteResponse>, Status> {
        let req = request.into_inner();

        if req.encrypted_cbor_payload.is_empty() {
            return Ok(Response::new(WriteResponse {
                success: false,
                journal_index: 0,
                timestamp_ms: 0,
                error_message: "Payload chiffré vide".into(),
            }));
        }

        Ok(Response::new(WriteResponse {
            success: true,
            journal_index: req.sequence_number,
            timestamp_ms: 1700000000000,
            error_message: "".into(),
        }))
    }

    async fn read_operation(
        &self,
        _request: Request<ReadRequest>,
    ) -> Result<Response<ReadResponse>, Status> {
        Ok(Response::new(ReadResponse {
            is_offline_read: false,
            encrypted_records: vec![vec![0xAA, 0xBB, 0xCC]],
            error_message: "".into(),
        }))
    }

    async fn get_cluster_status(
        &self,
        _request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        Ok(Response::new(StatusResponse {
            is_active: true,
            active_nodes_count: 3,
            read_only_mode: false,
            signed_license_token: vec![0x12, 0x34],
        }))
    }
}

#[tokio::test]
async fn test_grpc_client_mock_integration() {
    use ss_client::pb::amane_service_client::AmaneServiceClient;

    // 1. Démarrage du serveur Mock gRPC en arrière-plan sur 127.0.0.1:50051
    let addr = "127.0.0.1:50051".parse().unwrap();
    let mock_server = MockAmaneServer::default();

    tokio::spawn(async move {
        Server::builder()
            .add_service(AmaneServiceServer::new(mock_server))
            .serve(addr)
            .await
            .unwrap();
    });

    // Attente du démarrage du serveur Mock
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 2. Connexion du Client gRPC Rust (Mission B) au Mock Server
    let mut client = AmaneServiceClient::connect("http://127.0.0.1:50051")
        .await
        .unwrap();

    // 3. Test de la méthode WriteOperation
    let write_req = WriteRequest {
        epoch: 1,
        node_id: "NODE-TEST-01".into(),
        sequence_number: 101,
        op_type: "STOCK_UPDATE".into(),
        encrypted_cbor_payload: vec![0xDE, 0xAD, 0xBE, 0xEF],
    };

    let response = client.write_operation(write_req).await.unwrap().into_inner();
    assert!(response.success);
    assert_eq!(response.journal_index, 101);
}
