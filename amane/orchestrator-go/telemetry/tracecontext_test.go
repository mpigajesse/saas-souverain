package telemetry

import (
	"bytes"
	"context"
	"io"
	"log/slog"
	"strings"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

func TestParseTraceparentValid(t *testing.T) {
	h := "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
	tr, err := ParseTraceparent(h)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if tr.TraceID != "4bf92f3577b34da6a3ce929d0e0e4736" || tr.SpanID != "00f067aa0ba902b7" {
		t.Fatalf("ids mal extraits: %+v", tr)
	}
	if tr.String() != h {
		t.Fatalf("round-trip: %s != %s", tr.String(), h)
	}
}

func TestParseTraceparentInvalid(t *testing.T) {
	for _, h := range []string{"", "00-zz-zz-01", "01-deadbeef-01", "00-4bf92f3577b34da6a3ce929d0e0e473600f067aa0ba902b7-01"} {
		if tr, err := ParseTraceparent(h); err == nil {
			t.Fatalf("header invalide accepté: %q -> %+v", h, tr)
		}
	}
}

func TestRootAndChild(t *testing.T) {
	r := Root()
	if len(r.TraceID) != 32 || len(r.SpanID) != 16 {
		t.Fatalf("root mal formée: %+v", r)
	}
	c := r.Child()
	if c.TraceID != r.TraceID {
		t.Fatalf("child ne partage pas le trace-id: %s != %s", c.TraceID, r.TraceID)
	}
	if c.SpanID == r.SpanID {
		t.Fatal("child et parent partagent le même span-id")
	}
}

func TestContextCarriesTrace(t *testing.T) {
	tr := Root()
	ctx := ContextWithTrace(context.Background(), tr)
	if got := FromContext(ctx); got != tr {
		t.Fatalf("trace perdue au passage du contexte: %v != %v", got, tr)
	}
}

// TestServerClientRoundTrip : le client injecte le traceparent dans le metadata
// sortant ; le serveur l'extrait et le fait voir au handler (forward path
// nœud → nœud). Le handler voit un span enfant du trace émetteur.
func TestServerClientRoundTrip(t *testing.T) {
	clientInterceptor := UnaryClientInterceptor()
	serverInterceptor := UnaryServerInterceptor(slog.New(slog.NewTextHandler(io.Discard, nil)))

	want := Root()

	// Côté client : l'intercepteur attache le traceparent au metadata sortant.
	// (AppendToOutgoingContext rend un nouveau contexte — on lit celui que
	// reçoit l'invoker, c'est celui qui part sur le réseau.)
	var mdOut metadata.MD
	invoked := false
	_ = clientInterceptor(ContextWithTrace(context.Background(), want),
		"/amane.framework.v1.AmaneService/PushDelta",
		nil, nil, nil,
		func(ctx context.Context, method string, req, reply any, cc *grpc.ClientConn, opts ...grpc.CallOption) error {
			invoked = true
			mdOut, _ = metadata.FromOutgoingContext(ctx)
			return nil
		})

	if !invoked {
		t.Fatal("invoker client jamais appelé")
	}
	if len(mdOut.Get(traceparentKey)) != 1 {
		t.Fatalf("traceparent absent du metadata sortant: %v", mdOut)
	}
	if mdOut.Get(traceparentKey)[0] != want.String() {
		t.Fatalf("traceparent mal injecté: %s", mdOut.Get(traceparentKey)[0])
	}

	// Côté serveur : on transporte le metadata reçu dans le contexte entrant.
	recv := metadata.NewIncomingContext(context.Background(), mdOut)
	var seen *Trace
	_, _ = serverInterceptor(recv, nil, nil,
		func(ctx context.Context, req any) (any, error) {
			seen = FromContext(ctx)
			return nil, nil
		})

	if seen == nil {
		t.Fatal("le handler n'a pas vu de trace")
	}
	if seen.TraceID != want.TraceID {
		t.Fatalf("trace-id non propagé: %s != %s", seen.TraceID, want.TraceID)
	}
	if seen.SpanID == want.SpanID {
		t.Fatal("le handler doit porter un span enfant, pas le span émetteur")
	}
}

// TestServerGeneratesRootWhenAbsent : pas de traceparent entrant → le serveur
// crée une trace racine pour que les logs du nœud soient quand même corrélés.
func TestServerGeneratesRootWhenAbsent(t *testing.T) {
	si := UnaryServerInterceptor(slog.New(slog.NewTextHandler(io.Discard, nil)))
	var seen *Trace
	_, _ = si(context.Background(), nil, nil,
		func(ctx context.Context, req any) (any, error) {
			seen = FromContext(ctx)
			return nil, nil
		})
	if seen == nil || len(seen.TraceID) != 32 {
		t.Fatalf("trace racine non générée: %+v", seen)
	}
}

// TestParseTraceparentNonHex : longueur et séparateurs valides mais composants
// non hexadécimaux → refus (branche isHex false).
func TestParseTraceparentNonHex(t *testing.T) {
	h := "00-4bf92f3577b34da6a3ce929d0e0e473g-00f067aa0ba902b7-01"
	if tr, err := ParseTraceparent(h); err == nil {
		t.Fatalf("trace-id contenant 'g' accepté: %+v", tr)
	}
	h2 := "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b-01"
	if tr, err := ParseTraceparent(h2); err == nil {
		t.Fatalf("span-id trop court accepté: %+v", tr)
	}
}

// TestLogAttrsAndStartSpan : attributs slog de corrélation + fenêtre de timing
// (chemin failover/lease) — nil-safe et à la fois avec une trace.
func TestLogAttrsAndStartSpan(t *testing.T) {
	tr := Root()
	attrs := tr.LogAttrs()
	if len(attrs) != 4 || attrs[1] != tr.TraceID || attrs[3] != tr.SpanID {
		t.Fatalf("LogAttrs incohérents: %v", attrs)
	}

	var buf bytes.Buffer
	logger := slog.New(slog.NewTextHandler(&buf, nil))

	// Span sans trace (nil-safe).
	closeSpan := StartSpan(logger, "failover", nil)
	closeSpan("ok", "leader", "a")
	// Span corrélé à la trace courante.
	closeTrace := StartSpan(logger, "lease", tr)
	closeTrace("lost", "leader", tr.SpanID)

	out := buf.String()
	if !strings.Contains(out, "duration_ms") || !strings.Contains(out, "trace_id="+tr.TraceID) {
		t.Fatalf("span loggés incomplets:\n%s", out)
	}
}
