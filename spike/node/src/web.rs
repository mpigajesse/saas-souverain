use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const HTML: &str = r#"<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>EL BARAA CONSULT — Logiciel de gestion</title>
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
    :root {
      --crimson: #A01520;
      --gold: #C9A84C;
      --bg: #F7F5F2;
      --card: #FFFFFF;
      --text: #1A1A1A;
      --muted: #6B7280;
      --border: #E5E7EB;
    }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }

    /* ── Header ── */
    header {
      background: #1C1C1E;
      padding: 16px 32px;
      display: flex;
      align-items: center;
      justify-content: space-between;
    }
    .brand { display: flex; align-items: center; gap: 12px; }
    .brand-dot {
      width: 36px; height: 36px;
      background: var(--crimson);
      border-radius: 8px;
      display: flex; align-items: center; justify-content: center;
      font-weight: 900; color: #fff; font-size: 1rem;
    }
    .brand-name { color: #fff; font-weight: 700; font-size: 1rem; }
    .brand-sub { color: var(--gold); font-size: 0.75rem; font-weight: 500; }
    .header-badge {
      display: flex; align-items: center; gap: 6px;
      background: rgba(255,255,255,.08);
      border: 1px solid rgba(255,255,255,.12);
      border-radius: 20px;
      padding: 6px 14px;
      font-size: 0.78rem; color: rgba(255,255,255,.7);
    }
    .dot-green {
      width: 7px; height: 7px;
      background: #22C55E;
      border-radius: 50%;
      animation: pulse 2s infinite;
    }
    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: .4; }
    }

    /* ── Main ── */
    main {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 48px 24px;
    }
    .card {
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 20px;
      padding: 56px 48px;
      max-width: 520px;
      width: 100%;
      text-align: center;
      box-shadow: 0 8px 32px rgba(0,0,0,.06);
    }

    /* ── Icône animée ── */
    .icon-wrapper {
      width: 80px; height: 80px;
      background: linear-gradient(135deg, #FFF0F0, #FFF8F0);
      border: 2px solid #FECACA;
      border-radius: 20px;
      display: flex; align-items: center; justify-content: center;
      margin: 0 auto 28px;
      font-size: 2rem;
    }

    h1 {
      font-size: 1.5rem;
      font-weight: 800;
      margin-bottom: 10px;
      color: var(--text);
      line-height: 1.3;
    }
    h1 span { color: var(--crimson); }

    .subtitle {
      color: var(--muted);
      font-size: 0.9rem;
      line-height: 1.6;
      margin-bottom: 36px;
    }

    /* ── Feature list ── */
    .features {
      display: flex;
      flex-direction: column;
      gap: 10px;
      margin-bottom: 36px;
      text-align: left;
    }
    .feature-item {
      display: flex;
      align-items: flex-start;
      gap: 10px;
      padding: 12px 16px;
      background: #F9FAFB;
      border: 1px solid var(--border);
      border-radius: 10px;
      font-size: 0.875rem;
      color: var(--text);
    }
    .feature-item svg { flex-shrink: 0; margin-top: 1px; color: var(--crimson); }

    /* ── Sovereign badge ── */
    .sovereign {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      background: #F0FDF4;
      border: 1px solid #BBF7D0;
      border-radius: 10px;
      padding: 12px 20px;
      font-size: 0.82rem;
      color: #166534;
      font-weight: 500;
    }

    /* ── Node info ── */
    .node-info {
      margin-top: 24px;
      padding-top: 24px;
      border-top: 1px solid var(--border);
      display: flex;
      justify-content: center;
      gap: 24px;
      font-size: 0.78rem;
      color: var(--muted);
    }
    .node-info strong { color: var(--text); }

    @media (max-width: 600px) {
      .card { padding: 36px 24px; }
      header { padding: 12px 16px; }
    }
  </style>
</head>
<body>

  <header>
    <div class="brand">
      <div class="brand-dot">E</div>
      <div>
        <div class="brand-name">EL BARAA CONSULT</div>
        <div class="brand-sub">Logiciel de gestion PME</div>
      </div>
    </div>
    <div class="header-badge">
      <div class="dot-green"></div>
      Nœud actif — données sécurisées
    </div>
  </header>

  <main>
    <div class="card">

      <div class="icon-wrapper">📦</div>

      <h1>Gestion de <span>stock</span><br>& facturation</h1>

      <p class="subtitle">
        Votre logiciel de gestion est installé et opérationnel sur cette machine.
        L'interface complète est en cours de déploiement.
      </p>

      <div class="features">
        <div class="feature-item">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 1a7 7 0 1 1 0 14A7 7 0 0 1 8 1zm3.5 4.5-4.3 4.3-1.7-1.7-1 1 2.7 2.7 5.3-5.3-1-1z"/>
          </svg>
          <div>
            <strong>Gestion de stock</strong> — articles, entrées/sorties, alertes de seuil
          </div>
        </div>
        <div class="feature-item">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 1a7 7 0 1 1 0 14A7 7 0 0 1 8 1zm3.5 4.5-4.3 4.3-1.7-1.7-1 1 2.7 2.7 5.3-5.3-1-1z"/>
          </svg>
          <div>
            <strong>Facturation</strong> — devis, factures, suivi des paiements
          </div>
        </div>
        <div class="feature-item">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M8 1a7 7 0 1 1 0 14A7 7 0 0 1 8 1zm3.5 4.5-4.3 4.3-1.7-1.7-1 1 2.7 2.7 5.3-5.3-1-1z"/>
          </svg>
          <div>
            <strong>Synchronisation cluster</strong> — données répliquées sur vos machines
          </div>
        </div>
      </div>

      <div class="sovereign">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 1l1.5 4.5H14l-3.7 2.7 1.4 4.3L8 10 4.3 12.5l1.4-4.3L2 5.5h4.5L8 1z"/>
        </svg>
        Vos données restent sur VOS machines — chiffrées, jamais transmises
      </div>

      <div class="node-info">
        <span>Nœud <strong id="node-host">__NODE_ID__</strong></span>
        <span>Journal <strong>chiffré</strong> (XChaCha20)</span>
        <span>Cluster <strong>actif</strong></span>
      </div>

    </div>
  </main>

</body>
</html>
"#;

/// Démarre un serveur HTTP minimal sur le port donné.
/// Sert une page d'accueil statique pour le logiciel métier PME.
pub async fn serve(port: u16, node_id: uuid::Uuid) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            println!("  Web      : interface disponible sur http://0.0.0.0:{}", port);
            l
        }
        Err(e) => {
            println!("  Web      : impossible de démarrer sur :{} — {}", port, e);
            return;
        }
    };

    let html = HTML.replace("__NODE_ID__", &node_id.to_string()[..8]);
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );

    loop {
        let (mut stream, _peer) = match listener.accept().await {
            Ok(s) => s,
            Err(_) => continue,
        };
        let resp = response.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 512];
            stream.read(&mut buf).await.ok();
            stream.write_all(resp.as_bytes()).await.ok();
        });
    }
}
