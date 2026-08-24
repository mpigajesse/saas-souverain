package consensus

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"
)

const (
	defaultPrefix = "/amane/members/"
	leaderKey     = "/amane/leader"
	ttlSeconds    = 30
)

var (
	ErrMemberAlreadyExists = errors.New("member déjà enrôlé")
	ErrMemberNotFound      = errors.New("member introuvable")
)

// Member représente une machine enrôlée dans le cluster Amane (Interface 1).
// Ne contient que des identifiants et l'empreinte de la clé publique AK —
// jamais la clé privée.
type Member struct {
	MachineID     string    `json:"machine_id"`
	AKPublicKey   []byte    `json:"-"` // entrée uniquement (jamais persistée, jamais la privée)
	AKPublicKeyID string    `json:"ak_public_key_id"` // empreinte, jamais la clé
	SiteID        string    `json:"site_id"`
	Operator      string    `json:"operator,omitempty"`
	EnrolledAt    time.Time `json:"enrolled_at"`
	Revoked       bool      `json:"revoked"`
	RevokedAt     time.Time `json:"revoked_at,omitempty"`
}

// Fingerprint retourne une empreinte courte (8 premiers octets SHA-256) d'une
// clé publique. Utilisée comme identifiant de clé révoquée — jamais la clé elle-même.
func Fingerprint(publicKey []byte) string {
	sum := sha256.Sum256(publicKey)
	return hex.EncodeToString(sum[:8])
}

// Registry stocke le membership dans etcd (préfixe /amane/members/).
// Règle Interface 1 : posséder une clé (A) = appartenir au cluster (C).
type Registry struct {
	cli    *clientv3.Client
	logger *slog.Logger
	prefix string
}

func NewRegistry(cli *clientv3.Client, logger *slog.Logger) *Registry {
	return &Registry{cli: cli, logger: logger, prefix: defaultPrefix}
}

func (r *Registry) key(machineID string) string {
	return r.prefix + machineID
}

// Add enrôle une machine. Retourne le nouveau quorum après l'ajout.
func (r *Registry) Add(ctx context.Context, m Member) (int, error) {
	if m.MachineID == "" {
		return 0, errors.New("machine_id requis")
	}
	if len(m.AKPublicKey) == 0 {
		return 0, errors.New("ak_public_key requis")
	}
	m.AKPublicKeyID = Fingerprint(m.AKPublicKey)
	if m.EnrolledAt.IsZero() {
		m.EnrolledAt = time.Now().UTC()
	}
	data, err := json.Marshal(m)
	if err != nil {
		return 0, fmt.Errorf("encodage member: %w", err)
	}
	key := r.key(m.MachineID)

	// Txn : n'enrôle pas deux fois la même machine (idempotence stricte).
	tx, err := r.cli.Txn(ctx).
		If(clientv3.Compare(clientv3.CreateRevision(key), "=", 0)).
		Then(clientv3.OpPut(key, string(data))).
		Else(clientv3.OpGet(key)).
		Commit()
	if err != nil {
		return 0, fmt.Errorf("txn enrôlement: %w", err)
	}
	if !tx.Succeeded {
		return 0, fmt.Errorf("%w: %s", ErrMemberAlreadyExists, m.MachineID)
	}

	total, err := r.count(ctx)
	if err != nil {
		return 0, err
	}
	quorum := ComputeQuorum(total)
	r.logger.Info("machine enrôlée",
		"machine_id", m.MachineID,
		"ak_public_key_id", m.AKPublicKeyID,
		"site_id", m.SiteID,
		"quorum", quorum,
		"total", total,
	)
	return quorum, nil
}

// Remove révoque une machine et recalcule le quorum (dé-enrôlement < 1s,
// déclenché par NotifyRevocation depuis Mission A).
func (r *Registry) Remove(ctx context.Context, machineID string) (int, error) {
	if machineID == "" {
		return 0, errors.New("machine_id requis")
	}
	resp, err := r.cli.Delete(ctx, r.key(machineID))
	if err != nil {
		return 0, fmt.Errorf("suppression member: %w", err)
	}
	if resp.Deleted == 0 {
		return 0, fmt.Errorf("%w: %s", ErrMemberNotFound, machineID)
	}
	total, err := r.count(ctx)
	if err != nil {
		return 0, err
	}
	quorum := ComputeQuorum(total)
	r.logger.Warn("machine révoquée — quorum recalculé",
		"machine_id", machineID,
		"quorum", quorum,
		"total", total,
	)
	return quorum, nil
}

// Quorum retourne le quorum actuel sans modifier le membership.
func (r *Registry) Quorum(ctx context.Context) (int, error) {
	total, err := r.count(ctx)
	if err != nil {
		return 0, err
	}
	return ComputeQuorum(total), nil
}

func (r *Registry) count(ctx context.Context) (int, error) {
	resp, err := r.cli.Get(ctx, r.prefix, clientv3.WithPrefix())
	if err != nil {
		return 0, fmt.Errorf("comptage members: %w", err)
	}
	return len(resp.Kvs), nil
}

// Has vérifie la présence d'une machine (ex. recalcul de quorum sur élection).
func (r *Registry) Has(ctx context.Context, machineID string) (bool, error) {
	resp, err := r.cli.Get(ctx, r.key(machineID))
	if err != nil {
		return false, fmt.Errorf("lecture member: %w", err)
	}
	return resp.Count > 0, nil
}
