package consensus

import (
	"context"
	"testing"
	"time"
)

// TestWaitCtx — helper d'attente bornée du cycle élection : contexte annulé →
// retourne faux sans attendre la durée ; sinon vrai après la durée.
func TestWaitCtx(t *testing.T) {
	done := make(chan struct{})
	time.AfterFunc(100*time.Millisecond, func() { close(done) })
	ctx, cancel := context.WithCancel(context.Background())
	go func() {
		<-done
		cancel()
	}()
	if waitCtx(ctx, time.Hour) {
		t.Fatal("waitCtx: contexte annulé aurait dû rendre false")
	}

	// Durée courte écoulée sans annulation → true.
	if !waitCtx(context.Background(), 5*time.Millisecond) {
		t.Fatal("waitCtx: délai écoulé aurait dû rendre true")
	}
}
