package grpcserver

import (
	"crypto/tls"
	"crypto/x509"
	"fmt"
	"os"

	"google.golang.org/grpc/credentials"
)

// LoadClientCredentials construit des credentials TLS 1.3 pour un client gRPC
// (réplication inter-site, orchestration). Si certFile/keyFile sont fournis,
// active le mTLS : le client présente un certificat signé par la CA interne.
// serverName est le nom attendu dans le SAN du certificat serveur (SNI) —
// jamais InsecureSkipVerify : le modèle de confiance est la CA interne partagée.
func LoadClientCredentials(caFile, certFile, keyFile, serverName string) (credentials.TransportCredentials, error) {
	pem, err := os.ReadFile(caFile)
	if err != nil {
		return nil, fmt.Errorf("lecture du CA: %w", err)
	}
	pool := x509.NewCertPool()
	if !pool.AppendCertsFromPEM(pem) {
		return nil, fmt.Errorf("aucun certificat CA valide dans %s", caFile)
	}

	cfg := &tls.Config{
		RootCAs:    pool,
		ServerName: serverName,
		MinVersion: tls.VersionTLS13,
	}

	if certFile != "" || keyFile != "" {
		if certFile == "" || keyFile == "" {
			return nil, fmt.Errorf("certFile et keyFile doivent être fournis ensemble (mTLS)")
		}
		cert, err := tls.LoadX509KeyPair(certFile, keyFile)
		if err != nil {
			return nil, fmt.Errorf("chargement du certificat client: %w", err)
		}
		cfg.Certificates = []tls.Certificate{cert}
	}

	return credentials.NewTLS(cfg), nil
}
