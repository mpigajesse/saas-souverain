package grpcserver

import (
	"context"
	"log/slog"
	"runtime/debug"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"github.com/amane/orchestrator-go/telemetry"
)

// LoggingUnaryInterceptor journalise chaque appel RPC (méthode, code, latence)
// et, si le forward path a propagé un traceparent, les identifiants de trace
// (corrélation distribuée d'un nœud à l'autre). Latence en millisecondes —
// utile pour corréler le timing failover/lease (jalon 2).
func LoggingUnaryInterceptor(logger *slog.Logger) grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (any, error) {
		start := time.Now()
		resp, err := handler(ctx, req)
		st := status.Convert(err)
		attrs := []any{
			"method", info.FullMethod,
			"code", st.Code().String(),
			"duration_ms", float64(time.Since(start).Microseconds()) / 1000.0,
		}
		if tr := telemetry.FromContext(ctx); tr != nil {
			attrs = append(attrs, tr.LogAttrs()...)
		}
		logger.Info("rpc", attrs...)
		return resp, err
	}
}

// RecoveryUnaryInterceptor convertit un panic de handler en codes.Internal :
// le serveur ne tombe jamais à cause d'une méthode fautive.
func RecoveryUnaryInterceptor(logger *slog.Logger) grpc.UnaryServerInterceptor {
	return func(ctx context.Context, req any, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (out any, err error) {
		defer func() {
			if r := recover(); r != nil {
				logger.Error("panic dans le handler",
					"method", info.FullMethod,
					"panic", r,
					"stack", string(debug.Stack()),
				)
				err = status.Error(codes.Internal, "internal server panic")
			}
		}()
		return handler(ctx, req)
	}
}
