package consensus

import (
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"
)

// Keepalive gRPC du client etcd. Sans lui, la connexion vers un membre etcd
// isolé par une partition reste ouverte en silence : toutes les requêtes s'y
// bloquent jusqu'à leur délai et le client ne bascule jamais vers les membres
// sains. Constaté en laboratoire (ha-lab, refonte V3 étape 2a) : heartbeats,
// droit de forcer et libération du lock Patroni échouaient tous, et la
// bascule retombait sur le ttl de Patroni (~20 s).
//
// Le serveur etcd doit accepter ce rythme de sonde : --grpc-keepalive-min-time
// (ETCD_GRPC_KEEPALIVE_MIN_TIME) ≤ KeepAliveTime, sinon il ferme la connexion
// (« too_many_pings »). Sa valeur par défaut est 5 s.
const (
	KeepAliveTime    = 1 * time.Second
	KeepAliveTimeout = 1 * time.Second
)

// AutoSyncInterval : le client relit périodiquement la liste des membres etcd
// et s'en sert comme points d'entrée. Sur une machine PME, l'orchestrateur ne
// connaît au départ que l'etcd local ; si celui-ci tombe, il continue de
// joindre le cluster par les autres membres.
const AutoSyncInterval = 30 * time.Second

// NewClient ouvre un client etcd v3 (DCS du cluster Amane).
func NewClient(endpoints []string) (*clientv3.Client, error) {
	return clientv3.New(clientv3.Config{
		Endpoints:            endpoints,
		DialTimeout:          5 * time.Second,
		DialKeepAliveTime:    KeepAliveTime,
		DialKeepAliveTimeout: KeepAliveTimeout,
		AutoSyncInterval:     AutoSyncInterval,
	})
}
