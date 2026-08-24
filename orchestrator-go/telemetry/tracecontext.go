// Package telemetry fournit le chemin « forward » des traces distribué
// (jalon observabilité) : propagation W3C traceparent à travers les appels
// gRPC + corrélation dans les logs slog. Zéro dépendance externe : le format
// est celui d'OpenTelemetry, on peut donc brancher un vrai SDK OTel (export
// OTLP) sans changer les points d'ancrage.
package telemetry

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"log/slog"
	"time"
)

// Trace identifie une exécution distribuée (un write répliqué, un failover).
type Trace struct {
	TraceID string // 32 hex
	SpanID  string // 16 hex (parent courant)
}

// traceContextKey porte le Trace courant dans context.Context.
type traceContextKey struct{}

// FromContext retourne le Trace courant, ou nil s'il n'y en a pas.
func FromContext(ctx context.Context) *Trace {
	t, _ := ctx.Value(traceContextKey{}).(*Trace)
	return t
}

// ContextWithTrace enracine t dans ctx (et conserve les valeurs existantes).
func ContextWithTrace(ctx context.Context, t *Trace) context.Context {
	return context.WithValue(ctx, traceContextKey{}, t)
}

// Root crée une trace racine (nouveau trace-id, nouveau span-id).
func Root() *Trace {
	return &Trace{TraceID: randHex(16), SpanID: randHex(8)}
}

// Child crée un span enfant du trace courant : même trace-id, nouveau span-id.
func (t *Trace) Child() *Trace {
	return &Trace{TraceID: t.TraceID, SpanID: randHex(8)}
}

func randHex(n int) string {
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		// crypto/rand ne doit jamais échouer sous Unix ; secours horodaté.
		return hex.EncodeToString([]byte(time.Now().Format("150405.000000000")))
	}
	return hex.EncodeToString(b)
}

// ParseTraceparent décode une en-tête W3C traceparent au format
// "00-<trace-id 32>-<span-id 16>-<flags 2>". Retourne (nil, err) si invalide.
func ParseTraceparent(h string) (*Trace, error) {
	if len(h) != 55 {
		return nil, errors.New("traceparent: longueur invalide")
	}
	if h[0:2] != "00" || h[2] != '-' || h[35] != '-' || h[52] != '-' {
		return nil, errors.New("traceparent: format invalide")
	}
	t := &Trace{TraceID: h[3:35], SpanID: h[36:52]}
	if !isHex(t.TraceID) || !isHex(t.SpanID) {
		return nil, errors.New("traceparent: composants non hexadécimaux")
	}
	return t, nil
}

func isHex(s string) bool {
	for _, c := range s {
		if !((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F')) {
			return false
		}
	}
	return true
}

// String rend l'en-tête W3C (flags 01 = sampled).
func (t *Trace) String() string {
	return "00-" + t.TraceID + "-" + t.SpanID + "-01"
}

// LogAttrs rend les attributs slog de corrélation (trace_id, span_id).
func (t *Trace) LogAttrs() []any {
	return []any{"trace_id", t.TraceID, "span_id", t.SpanID}
}

// StartSpan ouvre une fenêtre de timing corrélée (pour le chemin critique
// failover/lease) et la journalise : utile au debug distribué multi-nœuds.
func StartSpan(logger *slog.Logger, name string, t *Trace) func(status string, extra ...any) {
	start := time.Now()
	return func(status string, extra ...any) {
		args := []any{"span", name, "status", status, "duration_ms",
			float64(time.Since(start).Microseconds()) / 1000.0}
		if t != nil {
			args = append(args, t.LogAttrs()...)
		}
		args = append(args, extra...)
		logger.Info("span", args...)
	}
}
