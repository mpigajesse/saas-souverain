package telemetry

import (
	"context"
	"log/slog"

	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

// traceparentKey est le nom de l'en-tête de propagation W3C dans le metadata gRPC.
const traceparentKey = "traceparent"

// UnaryServerInterceptor extrait le traceparent du metadata entrant, le place
// dans le contexte du handler (le forward path continue d'un nœud à l'autre)
// et génère une trace racine si l'appelant n'en portait pas. La méthode voit
// donc toujours une trace — les logs de chaque nœud partagent le même trace-id.
func UnaryServerInterceptor(logger *slog.Logger) grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		var tr *Trace
		if md, ok := metadata.FromIncomingContext(ctx); ok {
			if vals := md.Get(traceparentKey); len(vals) > 0 {
				if parsed, err := ParseTraceparent(vals[0]); err == nil {
					tr = parsed.Child()
				} else {
					logger.Debug("traceparent invalide ignoré", "header", vals[0], "err", err)
				}
			}
		}
		if tr == nil {
			tr = Root()
		}
		return handler(ContextWithTrace(ctx, tr), req)
	}
}

// UnaryClientInterceptor injecte le traceparent du contexte appelant dans le
// metadata sortant : la trace traverse la frontière du processus (PushDelta,
// Write vers le pair) — c'est le forward path entre les nœuds.
func UnaryClientInterceptor() grpc.UnaryClientInterceptor {
	return func(ctx context.Context, method string, req, reply any, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
		if tr := FromContext(ctx); tr != nil {
			ctx = metadata.AppendToOutgoingContext(ctx, traceparentKey, tr.String())
		}
		return invoker(ctx, method, req, reply, cc, opts...)
	}
}
