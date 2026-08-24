package grpcserver

import (
	"context"
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/pem"
	"log/slog"
	"math/big"
	"net"
	"os"
	"path/filepath"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"

	pb "github.com/amane/orchestrator-go/gen/amane/framework/v1"
)

// ---- PKI interne de test : CA + serveur (SAN 127.0.0.1) + clients (valide, sans cert, étranger) ----

type testPKI struct {
	dir string

	ca    string
	caKey string

	serverCert string
	serverKey  string

	clientCert string
	clientKey  string

	foreignCert string
	foreignKey  string

	caPool *x509.CertPool
	caCert *x509.Certificate
	caSign *ecdsa.PrivateKey
}

func newTestLogger() *slog.Logger {
	return slog.New(slog.NewTextHandler(discardWriter{}, nil))
}

// issue délivre un certificat feuille (serveur ou client) signé par une CA donnée.
// En mode serveur, le SAN porte l'adresse IP 127.0.0.1 vérifiée au handshake.
func issue(t *testing.T, ca *x509.Certificate, caKey *ecdsa.PrivateKey, cn string, isServer bool) (*x509.Certificate, *ecdsa.PrivateKey) {
	t.Helper()
	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	eku := []x509.ExtKeyUsage{x509.ExtKeyUsageClientAuth}
	if isServer {
		eku = []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth}
	}
	tmpl := &x509.Certificate{
		SerialNumber: big.NewInt(time.Now().UnixNano()),
		Subject:      pkix.Name{CommonName: cn},
		NotBefore:    time.Now().Add(-time.Hour),
		NotAfter:     time.Now().Add(24 * time.Hour),
		KeyUsage:     x509.KeyUsageDigitalSignature | x509.KeyUsageKeyEncipherment,
		ExtKeyUsage:  eku,
	}
	if isServer {
		tmpl.IPAddresses = []net.IP{net.ParseIP("127.0.0.1")}
		tmpl.DNSNames = []string{"localhost"}
	}
	der, err := x509.CreateCertificate(rand.Reader, tmpl, ca, &key.PublicKey, caKey)
	if err != nil {
		t.Fatal(err)
	}
	cert, err := x509.ParseCertificate(der)
	if err != nil {
		t.Fatal(err)
	}
	return cert, key
}

func newCA(t *testing.T) (*x509.Certificate, *ecdsa.PrivateKey) {
	t.Helper()
	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	ca := &x509.Certificate{
		SerialNumber:          big.NewInt(1),
		Subject:               pkix.Name{CommonName: "amane-internal-ca"},
		NotBefore:             time.Now().Add(-time.Hour),
		NotAfter:              time.Now().Add(48 * time.Hour),
		IsCA:                  true,
		KeyUsage:              x509.KeyUsageCertSign | x509.KeyUsageCRLSign,
		BasicConstraintsValid: true,
	}
	der, err := x509.CreateCertificate(rand.Reader, ca, ca, &key.PublicKey, key)
	if err != nil {
		t.Fatal(err)
	}
	cert, err := x509.ParseCertificate(der)
	if err != nil {
		t.Fatal(err)
	}
	return cert, key
}

func writeKeyPair(t *testing.T, dir, name string, cert *x509.Certificate, key *ecdsa.PrivateKey) {
	t.Helper()
	os.WriteFile(filepath.Join(dir, name+".crt"), pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: cert.Raw}), 0o600)
	keyDER, err := x509.MarshalECPrivateKey(key)
	if err != nil {
		t.Fatal(err)
	}
	os.WriteFile(filepath.Join(dir, name+".key"), pem.EncodeToMemory(&pem.Block{Type: "EC PRIVATE KEY", Bytes: keyDER}), 0o600)
}

func newTestPKI(t *testing.T) testPKI {
	t.Helper()
	dir := t.TempDir()
	caCert, caKey := newCA(t)

	serverCert, serverKey := issue(t, caCert, caKey, "orchestrator.amane.internal", true)
	clientCert, clientKey := issue(t, caCert, caKey, "node-1.amane.internal", false)

	foreignCA, foreignCAKey := newCA(t)
	foreignCert, foreignKey := issue(t, foreignCA, foreignCAKey, "intruder.amane.internal", false)

	writeKeyPair(t, dir, "ca", caCert, caKey)
	writeKeyPair(t, dir, "server", serverCert, serverKey)
	writeKeyPair(t, dir, "client", clientCert, clientKey)
	writeKeyPair(t, dir, "foreign", foreignCert, foreignKey)

	pool := x509.NewCertPool()
	pool.AddCert(caCert)

	return testPKI{
		dir:         dir,
		ca:          filepath.Join(dir, "ca.crt"),
		serverCert:  filepath.Join(dir, "server.crt"),
		serverKey:   filepath.Join(dir, "server.key"),
		clientCert:  filepath.Join(dir, "client.crt"),
		clientKey:   filepath.Join(dir, "client.key"),
		foreignCert: filepath.Join(dir, "foreign.crt"),
		foreignKey:  filepath.Join(dir, "foreign.key"),
		caPool:      pool,
		caCert:      caCert,
		caSign:      caKey,
	}
}

func startTLSGRPCServer(t *testing.T, pki testPKI) (addr string, stop func()) {
	t.Helper()
	creds, err := LoadServerCredentials(pki.serverCert, pki.serverKey, pki.ca)
	if err != nil {
		t.Fatalf("credentials serveur: %v", err)
	}
	lis, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	s := grpc.NewServer(grpc.Creds(creds))
	pb.RegisterAmaneServiceServer(s, New(newTestLogger()))
	go func() {
		if err := s.Serve(lis); err != nil {
			t.Errorf("serve: %v", err)
		}
	}()
	t.Cleanup(s.Stop)
	return lis.Addr().String(), s.Stop
}

// ping appelle l'RPC Ping avec un timeout — le handshake TLS a lieu à la connexion.
func ping(ctx context.Context, conn *grpc.ClientConn) error {
	_, err := pb.NewAmaneServiceClient(conn).Ping(ctx, &pb.PingRequest{})
	return err
}

func dialSrv(t *testing.T, addr string, c credentials.TransportCredentials) *grpc.ClientConn {
	t.Helper()
	conn, err := grpc.NewClient(addr, grpc.WithTransportCredentials(c))
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	t.Cleanup(func() { conn.Close() })
	return conn
}

// TestCredentialsE2E_mTLS : authentification mutuelle, client valide → Ping ok.
func TestCredentialsE2E_mTLS(t *testing.T) {
	pki := newTestPKI(t)
	addr, _ := startTLSGRPCServer(t, pki)

	creds, err := LoadClientCredentials(pki.ca, pki.clientCert, pki.clientKey, "127.0.0.1")
	if err != nil {
		t.Fatalf("credentials client: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := ping(ctx, dialSrv(t, addr, creds)); err != nil {
		t.Fatalf("mTLS Ping a échoué: %v", err)
	}
}

// TestCredentialsE2E_noClientCert : sans certificat client, le handshake échoue (mTLS exigé).
func TestCredentialsE2E_noClientCert(t *testing.T) {
	pki := newTestPKI(t)
	addr, _ := startTLSGRPCServer(t, pki)

	creds, err := LoadClientCredentials(pki.ca, "", "", "127.0.0.1")
	if err != nil {
		t.Fatalf("credentials client: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := ping(ctx, dialSrv(t, addr, creds)); err == nil {
		t.Fatal("Ping a réussi alors qu'aucun certificat client n'était présenté (mTLS requis)")
	}
}

// TestCredentialsE2E_untrustedClient : un certificat étranger (autre CA) est refusé.
func TestCredentialsE2E_untrustedClient(t *testing.T) {
	pki := newTestPKI(t)
	addr, _ := startTLSGRPCServer(t, pki)

	_, _ = pki.caPool, pki.caCert // CA interne réservée aux certificats de confiance
	creds, err := LoadClientCredentials(pki.ca, pki.foreignCert, pki.foreignKey, "127.0.0.1")
	if err != nil {
		t.Fatalf("credentials client: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := ping(ctx, dialSrv(t, addr, creds)); err == nil {
		t.Fatal("Ping a réussi avec un certificat client signé par une CA étrangère")
	}
}

// TestCredentialsE2E_rejectTLS12 : le serveur refuse les handshakes en dessous de TLS 1.3.
func TestCredentialsE2E_rejectTLS12(t *testing.T) {
	pki := newTestPKI(t)
	addr, _ := startTLSGRPCServer(t, pki)

	creds := credentials.NewTLS(&tls.Config{
		RootCAs:    pki.caPool,
		ServerName: "127.0.0.1",
		MinVersion: tls.VersionTLS12,
		MaxVersion: tls.VersionTLS12,
	})
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := ping(ctx, dialSrv(t, addr, creds)); err == nil {
		t.Fatal("Ping a réussi en TLS 1.2 alors que MinVersion=1.3 est exigé")
	}
}

// TestLoadClientCredentials_validation : cohérence des arguments (mTLS par paires).
func TestLoadClientCredentials_validation(t *testing.T) {
	pki := newTestPKI(t)

	if _, err := LoadClientCredentials(pki.ca, pki.clientCert, "", "127.0.0.1"); err == nil {
		t.Fatal("certFile sans keyFile aurait dû être rejeté")
	}
	if _, err := LoadClientCredentials(filepath.Join(pki.dir, "absente.crt"), "", "", "127.0.0.1"); err == nil {
		t.Fatal("CA absent aurait dû être rejeté")
	}
}
