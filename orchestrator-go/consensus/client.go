package consensus

import (
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"
)

// NewClient ouvre un client etcd v3 (DCS du cluster Amane).
func NewClient(endpoints []string) (*clientv3.Client, error) {
	return clientv3.New(clientv3.Config{
		Endpoints:   endpoints,
		DialTimeout: 5 * time.Second,
	})
}