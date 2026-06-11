use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::header,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{AppState, User};

// ── HTML escaping ─────────────────────────────────────────────────────────────

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn esc_opt(s: &Option<String>) -> String {
    s.as_deref().map(esc).unwrap_or_default()
}

// ── CSS ───────────────────────────────────────────────────────────────────────

const CSS: &str = r#"
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
:root{--cr:#A01520;--go:#C9A84C;--bg:#F4F4F6;--card:#fff;--tx:#1A1A1A;--mu:#6B7280;--bd:#E5E7EB;--sb:#161618;--sb2:#222224}
body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;background:var(--bg);color:var(--tx);display:flex;min-height:100vh}

/* ── Sidebar ── */
.sidebar{width:240px;min-height:100vh;background:var(--sb);display:flex;flex-direction:column;position:fixed;left:0;top:0;bottom:0;z-index:200}
.sb-brand{padding:20px 18px 16px;border-bottom:1px solid rgba(255,255,255,.07);display:flex;align-items:center;gap:10px}
.sb-sq{width:34px;height:34px;background:var(--cr);border-radius:8px;display:flex;align-items:center;justify-content:center;color:#fff;font-weight:900;font-size:.95rem;flex-shrink:0}
.sb-name{color:#fff;font-weight:700;font-size:.9rem;line-height:1.2}
.sb-sub{color:var(--go);font-size:.68rem;margin-top:1px}
.sb-section{padding:18px 12px 6px;font-size:.65rem;font-weight:700;text-transform:uppercase;letter-spacing:.1em;color:rgba(255,255,255,.3)}
.sb-nav{display:flex;flex-direction:column;gap:2px;padding:0 10px}
.sb-nav a{display:flex;align-items:center;gap:10px;color:rgba(255,255,255,.55);text-decoration:none;font-size:.85rem;padding:9px 10px;border-radius:8px;transition:all .15s;font-weight:500}
.sb-nav a:hover{color:#fff;background:rgba(255,255,255,.07)}
.sb-nav a.active{color:#fff;background:var(--cr)}
.sb-nav a svg{flex-shrink:0;opacity:.8}
.sb-nav a.active svg{opacity:1}
.sb-footer{margin-top:auto;padding:14px 16px;border-top:1px solid rgba(255,255,255,.07)}
.sb-user{font-size:.75rem;color:rgba(255,255,255,.55);margin-bottom:8px;line-height:1.5}
.sb-user strong{color:#fff;display:block;font-size:.82rem}
.sb-role{display:inline-block;padding:2px 8px;border-radius:20px;font-size:.65rem;font-weight:700;margin-bottom:6px}
.sb-role.admin{background:rgba(201,168,76,.15);color:var(--go)}
.sb-role.employee{background:rgba(255,255,255,.07);color:rgba(255,255,255,.4)}
.btn-logout{background:rgba(255,255,255,.07);border:1px solid rgba(255,255,255,.1);color:rgba(255,255,255,.5);border-radius:6px;padding:5px 12px;font-size:.75rem;cursor:pointer;font-family:inherit;width:100%;transition:all .15s}
.btn-logout:hover{background:rgba(160,21,32,.4);border-color:rgba(160,21,32,.6);color:#fff}
.sb-dot{display:inline-block;width:6px;height:6px;background:#22C55E;border-radius:50%;margin-right:5px;animation:pulse 2s infinite}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:.3}}

/* ── Main ── */
.main{margin-left:240px;flex:1;min-height:100vh;display:flex;flex-direction:column}
.topbar{background:var(--card);border-bottom:1px solid var(--bd);padding:0 32px;height:56px;display:flex;align-items:center;justify-content:space-between;position:sticky;top:0;z-index:100}
.topbar-title{font-size:1rem;font-weight:700}
.topbar-right{display:flex;align-items:center;gap:10px;font-size:.78rem;color:var(--mu)}
.container{padding:28px 32px;flex:1}

/* ── Typography ── */
.page-title{font-size:1.35rem;font-weight:800;margin-bottom:4px}
.page-sub{color:var(--mu);font-size:.875rem;margin-bottom:24px}
.row-split{display:flex;align-items:center;justify-content:space-between;margin-bottom:24px}

/* ── Stats ── */
.stats{display:grid;grid-template-columns:repeat(3,1fr);gap:16px;margin-bottom:24px}
.stats-2{grid-template-columns:repeat(2,1fr)}
.sc{background:var(--card);border:1px solid var(--bd);border-radius:12px;padding:18px 22px}
.sl{font-size:.7rem;font-weight:700;text-transform:uppercase;letter-spacing:.07em;color:var(--mu);margin-bottom:8px}
.sv{font-size:1.9rem;font-weight:800}
.sv.danger{color:var(--cr)}
.sv.ok-val{color:#059669}

/* ── Panel ── */
.panel{background:var(--card);border:1px solid var(--bd);border-radius:12px;margin-bottom:20px;overflow:hidden}
.ph{padding:14px 20px;border-bottom:1px solid var(--bd);display:flex;align-items:center;justify-content:space-between}
.pt{font-weight:700;font-size:.9rem}

/* ── Table ── */
table{width:100%;border-collapse:collapse}
th{text-align:left;padding:9px 16px;font-size:.7rem;font-weight:700;text-transform:uppercase;letter-spacing:.05em;color:var(--mu);background:#F9FAFB;border-bottom:1px solid var(--bd)}
td{padding:11px 16px;font-size:.855rem;border-bottom:1px solid var(--bd);vertical-align:middle}
tr:last-child td{border-bottom:none}
tbody tr:hover td{background:#F9FAFB}
.mono{font-family:monospace;font-size:.8rem}

/* ── Badges ── */
.badge{display:inline-flex;align-items:center;border-radius:20px;padding:2px 10px;font-size:.7rem;font-weight:700}
.ok{background:#D1FAE5;color:#065F46}
.alerte{background:#FEE2E2;color:#991B1B}
.badge-admin{background:rgba(201,168,76,.15);color:#92600A}
.badge-emp{background:#F3F4F6;color:#4B5563}
.badge-off{background:#FEF2F2;color:#991B1B}

/* ── Buttons ── */
.btn{display:inline-flex;align-items:center;gap:6px;border:none;border-radius:8px;padding:8px 16px;font-size:.85rem;font-weight:600;cursor:pointer;text-decoration:none;transition:all .15s;font-family:inherit}
.btn-p{background:var(--cr);color:#fff}
.btn-p:hover{background:#8B1119;color:#fff}
.btn-sm{padding:4px 10px;font-size:.76rem}
.btn-g{background:transparent;border:1px solid var(--bd);color:var(--mu)}
.btn-g:hover{border-color:var(--tx);color:var(--tx)}
.btn-d{background:#FEE2E2;color:#991B1B;border:1px solid #FECACA}
.btn-d:hover{background:#FECACA}
.btn-y{background:#FEF3C7;color:#92400E;border:1px solid #FCD34D}
.btn-y:hover{background:#FCD34D}

/* ── Forms ── */
.form-grid{display:grid;grid-template-columns:1fr 1fr;gap:16px}
.fg{display:flex;flex-direction:column;gap:6px}
.full{grid-column:1/-1}
label{font-size:.81rem;font-weight:600}
input,select,textarea{border:1px solid var(--bd);border-radius:8px;padding:8px 12px;font-size:.875rem;font-family:inherit;width:100%;background:#fff;transition:border-color .15s}
input:focus,select:focus,textarea:focus{outline:none;border-color:var(--cr);box-shadow:0 0 0 3px rgba(160,21,32,.1)}
textarea{min-height:72px;resize:vertical}
.fa{display:flex;gap:10px;margin-top:8px}

/* ── Cluster status ── */
.cluster-card{background:var(--card);border:1px solid var(--bd);border-radius:12px;padding:20px 24px;margin-bottom:20px}
.cluster-role{display:flex;align-items:center;gap:12px;margin-bottom:14px}
.role-badge{padding:4px 14px;border-radius:20px;font-size:.8rem;font-weight:700}
.role-primary{background:#D1FAE5;color:#065F46}
.role-standby{background:#FEF3C7;color:#92400E}
.cluster-detail{font-size:.82rem;color:var(--mu);line-height:1.8}
.cluster-detail strong{color:var(--tx)}

/* ── Misc ── */
.empty{text-align:center;padding:36px 24px;color:var(--mu);font-size:.875rem}
.alert-bar{background:#FEF3C7;border:1px solid #FCD34D;border-radius:10px;padding:11px 16px;font-size:.84rem;color:#92400E;margin-bottom:18px;display:flex;align-items:center;gap:8px}
.success-bar{background:#D1FAE5;border:1px solid #6EE7B7;border-radius:10px;padding:11px 16px;font-size:.84rem;color:#065F46;margin-bottom:18px}
.err-bar{background:#FEE2E2;border:1px solid #FECACA;border-radius:10px;padding:11px 16px;font-size:.84rem;color:#991B1B;margin-bottom:18px}

/* ── Login page ── */
body.login-body{display:flex;align-items:center;justify-content:center;min-height:100vh;background:var(--bg)}
.login-card{background:var(--card);border:1px solid var(--bd);border-radius:16px;padding:36px;width:100%;max-width:380px;box-shadow:0 4px 24px rgba(0,0,0,.07)}
.login-logo{display:flex;align-items:center;gap:12px;margin-bottom:28px}
.login-name{font-weight:800;font-size:1.05rem;color:var(--tx)}
.login-sub{font-size:.72rem;color:var(--go);margin-top:1px}
.login-card .fg{margin-bottom:14px}
.login-err{background:#FEE2E2;border:1px solid #FECACA;border-radius:8px;padding:9px 14px;font-size:.82rem;color:#991B1B;margin-bottom:12px}

/* ── Responsive ── */
@media(max-width:900px){.sidebar{width:200px}.main{margin-left:200px}.container{padding:20px 18px}}
@media(max-width:640px){.sidebar{display:none}.main{margin-left:0}.stats{grid-template-columns:1fr}.form-grid{grid-template-columns:1fr}}
"#;

// ── Shared layout ─────────────────────────────────────────────────────────────

pub(crate) fn layout(title: &str, active: &str, user: &User, tenant: &str, content: &str) -> String {
    let is_admin = user.role == "admin";
    let tenant_esc = esc(tenant);
    let initial = tenant_esc.chars().next().unwrap_or('P').to_uppercase().to_string();
    let username = esc(&user.full_name);
    let role_label = if is_admin { "Administrateur" } else { "Employé" };
    let role_class = if is_admin { "admin" } else { "employee" };

    let da = if active == "dashboard" { "active" } else { "" };
    let aa = if active == "articles" { "active" } else { "" };
    let ma = if active == "mouvements" { "active" } else { "" };
    let ca = if active == "cluster" { "active" } else { "" };
    let ua = if active == "utilisateurs" { "active" } else { "" };

    let admin_section = if is_admin {
        format!(
            r#"
  <div class="sb-section">Administration</div>
  <nav class="sb-nav">
    <a href="/admin" class="{ca}">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
      </svg>
      Cluster
    </a>
    <a href="/admin/utilisateurs" class="{ua}">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/>
      </svg>
      Utilisateurs
    </a>
  </nav>"#,
            ca = ca,
            ua = ua,
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>{title} — {tenant_esc}</title>
<style>{CSS}</style>
</head>
<body>

<aside class="sidebar">
  <div class="sb-brand">
    <div class="sb-sq">{initial}</div>
    <div>
      <div class="sb-name">{tenant_esc}</div>
      <div class="sb-sub">Gestion PME</div>
    </div>
  </div>

  <div class="sb-section">Gestion de stock</div>
  <nav class="sb-nav">
    <a href="/" class="{da}">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
        <rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
      </svg>
      Tableau de bord
    </a>
    <a href="/articles" class="{aa}">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 8V21H3V8"/><path d="M23 3H1v5h22V3z"/><path d="M10 12h4"/>
      </svg>
      Articles
    </a>
    <a href="/mouvements" class="{ma}">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="23 6 13.5 15.5 8.5 10.5 1 18"/>
        <polyline points="17 6 23 6 23 12"/>
      </svg>
      Mouvements
    </a>
  </nav>
{admin_section}
  <div class="sb-footer">
    <div class="sb-user">
      <strong>{username}</strong>
      <span class="sb-role {role_class}">{role_label}</span>
    </div>
    <form method="post" action="/logout">
      <button type="submit" class="btn-logout">Déconnexion</button>
    </form>
  </div>
</aside>

<div class="main">
  <div class="topbar">
    <span class="topbar-title">{title}</span>
    <div class="topbar-right"><span class="sb-dot"></span>{tenant_esc}</div>
  </div>
  <div class="container">
{content}
  </div>
</div>

</body>
</html>"#
    )
}

// ── Standby detection ─────────────────────────────────────────────────────────

async fn primary_web_url(state: &Arc<AppState>) -> Option<String> {
    let host: Option<String> = sqlx::query_scalar(
        "SELECT sender_host FROM pg_stat_wal_receiver LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    host.map(|h| format!("http://{}:{}/login", h, state.web_port))
}

fn standby_html(tenant: &str, primary_url: Option<&str>) -> String {
    let tenant_esc = esc(tenant);
    let initial = tenant_esc.chars().next().unwrap_or('P').to_uppercase().to_string();
    let btn = match primary_url {
        Some(url) => format!(
            r#"<a href="{url}" class="btn btn-p" style="width:100%;justify-content:center;margin-top:8px">
              Accéder au nœud primaire →
            </a>"#,
            url = esc(url)
        ),
        None => String::new(),
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>Nœud secondaire — {tenant_esc}</title>
<style>{CSS}</style>
</head>
<body class="login-body">
<div class="login-card" style="max-width:420px;text-align:center">
  <div class="login-logo" style="justify-content:center;margin-bottom:20px">
    <div class="sb-sq">{initial}</div>
    <div>
      <div class="login-name">{tenant_esc}</div>
      <div class="login-sub">Nœud secondaire</div>
    </div>
  </div>
  <div class="alert-bar" style="text-align:left;margin-bottom:16px">
    ⚠️ Ce nœud est en mode <strong>standby</strong> (lecture seule).
  </div>
  <p style="font-size:.875rem;color:#4B5563;line-height:1.6;margin-bottom:4px">
    La connexion et les opérations d'écriture sont réservées au nœud primaire.
    Ce nœud prend le relais automatiquement en cas de panne.
  </p>
  {btn}
</div>
</body>
</html>"#
    )
}

// ── Login ─────────────────────────────────────────────────────────────────────

pub async fn login_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let is_standby = sqlx::query_scalar::<_, bool>("SELECT pg_is_in_recovery()")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(false);
    if is_standby {
        let url = primary_web_url(&state).await;
        return Html(standby_html(&state.tenant_name, url.as_deref()));
    }
    Html(login_html(&state.tenant_name, None))
}

fn login_html(tenant: &str, error: Option<&str>) -> String {
    let tenant_esc = esc(tenant);
    let initial = tenant_esc.chars().next().unwrap_or('P').to_uppercase().to_string();
    let err_block = match error {
        Some(msg) => format!(r#"<div class="login-err">⚠️ {}</div>"#, esc(msg)),
        None => String::new(),
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>Connexion — {tenant_esc}</title>
<style>{CSS}</style>
</head>
<body class="login-body">
<div class="login-card">
  <div class="login-logo">
    <div class="sb-sq">{initial}</div>
    <div>
      <div class="login-name">{tenant_esc}</div>
      <div class="login-sub">Gestion PME</div>
    </div>
  </div>
  {err_block}
  <form method="post" action="/login">
    <div class="fg">
      <label for="username">Identifiant</label>
      <input id="username" name="username" required autocomplete="username" placeholder="Votre identifiant">
    </div>
    <div class="fg">
      <label for="password">Mot de passe</label>
      <input id="password" name="password" type="password" required autocomplete="current-password" placeholder="••••••••">
    </div>
    <div class="fa" style="margin-top:4px">
      <button type="submit" class="btn btn-p" style="width:100%;justify-content:center">Se connecter</button>
    </div>
  </form>
</div>
</body>
</html>"#
    )
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login_post(
    State(state): State<Arc<AppState>>,
    Form(f): Form<LoginForm>,
) -> Response {
    let is_standby = sqlx::query_scalar::<_, bool>("SELECT pg_is_in_recovery()")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(false);
    if is_standby {
        let url = primary_web_url(&state).await;
        return Html(standby_html(&state.tenant_name, url.as_deref())).into_response();
    }

    let username = f.username.trim();
    let password = f.password.as_str();

    match super::authenticate(&state.pool, username, password).await {
        Some(user) => {
            let redirect_to = if user.role == "admin" { "/admin" } else { "/" };
            match super::create_session(&state.pool, user.id).await {
                Ok(sid) => (
                    [(header::SET_COOKIE, super::session_cookie_set(sid))],
                    Redirect::to(redirect_to),
                )
                    .into_response(),
                Err(_) => Html(login_html(
                    &state.tenant_name,
                    Some("Erreur interne — réessayez."),
                ))
                .into_response(),
            }
        }
        None => Html(login_html(
            &state.tenant_name,
            Some("Identifiant ou mot de passe incorrect."),
        ))
        .into_response(),
    }
}

// Logout is served from the authenticated routes group (auth middleware must pass first).
// On expired session the middleware already redirects to /login, so no special case needed.
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Response {
    // Delete all sessions for this user (logs out all devices)
    let _ = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user.id)
        .execute(&state.pool)
        .await;

    (
        [(header::SET_COOKIE, super::session_cookie_clear())],
        Redirect::to("/login"),
    )
        .into_response()
}

// ── Admin dashboard ───────────────────────────────────────────────────────────

pub async fn admin_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Html<String> {
    let cluster = super::get_cluster_info(&state.pool).await;

    let (role_label, role_class, replication_info) = if cluster.is_primary {
        let sb_count = cluster.standbys.len();
        let repl = if sb_count == 0 {
            r#"<span style="color:#92400E">⚠ Aucun nœud secondaire connecté</span>"#.to_string()
        } else {
            format!(
                r#"<span style="color:#065F46">✓ {} nœud(s) secondaire(s) connecté(s)</span>"#,
                sb_count
            )
        };
        ("Primaire", "role-primary", repl)
    } else {
        let lag = cluster.replication_lag.as_deref().unwrap_or("inconnue");
        let host = esc_opt(&cluster.primary_host);
        let repl = format!(
            r#"Réplique du nœud <strong>{}</strong> — état : <strong>{}</strong>"#,
            if host.is_empty() { "inconnu".to_string() } else { host },
            lag
        );
        ("Secondaire", "role-standby", repl)
    };

    let standbys_table = if cluster.is_primary && !cluster.standbys.is_empty() {
        let rows: String = cluster.standbys.iter().map(|s| {
            let addr = match &s.client_addr {
                Some(ip) => format!(r#"<span class="mono">{}</span>"#, esc(ip)),
                None => r#"<span style="color:var(--mu)">—</span>"#.to_string(),
            };
            format!(
                r#"<tr>
  <td>{addr}</td>
  <td class="mono" style="color:var(--mu);font-size:.78rem">{name}</td>
  <td><span class="badge ok">{state}</span></td>
</tr>"#,
                name = esc(&s.name),
                state = esc(&s.state),
            )
        }).collect();
        format!(
            r#"<table style="margin-top:14px">
<thead><tr><th>Adresse IP</th><th>Application</th><th>État réplication</th></tr></thead>
<tbody>{rows}</tbody></table>"#
        )
    } else if cluster.is_primary {
        r#"<p style="margin-top:10px;font-size:.83rem;color:#92400E">⚠ Aucun nœud secondaire connecté — bascule manuelle uniquement.</p>"#.to_string()
    } else {
        String::new()
    };

    let content = format!(
        r#"<div class="page-title">Cluster</div>
<div class="page-sub">État du cluster PostgreSQL de ce nœud</div>

<div class="cluster-card">
  <div class="cluster-role">
    <span class="role-badge {role_class}">{role_label}</span>
    <span style="font-weight:700;font-size:.95rem">Ce nœud</span>
  </div>
  <div class="cluster-detail">
    {replication_info}
    {standbys_table}
  </div>
</div>"#,
    );

    Html(layout("Cluster", "cluster", &user, &state.tenant_name, &content))
}

// ── Admin — utilisateurs ──────────────────────────────────────────────────────

pub async fn users_list(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Html<String> {
    let users = match super::list_users(&state.pool).await {
        Ok(v) => v,
        Err(e) => {
            return Html(layout(
                "Utilisateurs",
                "utilisateurs",
                &user,
                &state.tenant_name,
                &format!(r#"<div class="err-bar">Erreur : {}</div>"#, esc(&e.to_string())),
            ));

        }
    };

    let rows: String = users
        .iter()
        .map(|u| {
            let role_badge = if u.role == "admin" {
                r#"<span class="badge badge-admin">Admin</span>"#
            } else {
                r#"<span class="badge badge-emp">Employé</span>"#
            };
            let status_badge = if u.is_active {
                r#"<span class="badge ok">Actif</span>"#
            } else {
                r#"<span class="badge badge-off">Inactif</span>"#
            };
            let toggle_btn = if u.role != "admin" {
                let (btn_class, btn_label) = if u.is_active {
                    ("btn-d", "Désactiver")
                } else {
                    ("btn-y", "Activer")
                };
                format!(
                    r#"<form method="post" action="/admin/utilisateurs/{id}/toggle" style="display:inline">
  <button type="submit" class="btn {btn_class} btn-sm">{btn_label}</button>
</form>"#,
                    id = u.id,
                )
            } else {
                String::new()
            };
            let mdp_btn = format!(
                r#"<a href="/admin/utilisateurs/{id}/mdp" class="btn btn-g btn-sm">Mot de passe</a>"#,
                id = u.id,
            );
            format!(
                r#"<tr>
  <td class="mono">{username}</td>
  <td>{full_name}</td>
  <td>{role_badge}</td>
  <td>{status_badge}</td>
  <td style="color:var(--mu);font-size:.78rem">{date}</td>
  <td style="display:flex;gap:6px;flex-wrap:wrap">{mdp_btn}{toggle_btn}</td>
</tr>"#,
                username = esc(&u.username),
                full_name = esc(&u.full_name),
                date = u.created_at.format("%d/%m/%Y").to_string(),
            )
        })
        .collect();

    let active_count = users.iter().filter(|u| u.is_active).count();
    let admin_count  = users.iter().filter(|u| u.role == "admin" && u.is_active).count();
    let emp_count    = users.iter().filter(|u| u.role == "employee" && u.is_active).count();

    let content = format!(
        r#"<div class="row-split">
  <div>
    <div class="page-title">Utilisateurs</div>
    <div class="page-sub">{n} compte(s) enregistré(s)</div>
  </div>
  <a href="/admin/utilisateurs/nouveau" class="btn btn-p">+ Nouvel utilisateur</a>
</div>

<div class="stats" style="margin-bottom:24px">
  <div class="sc">
    <div class="sl">Comptes actifs</div>
    <div class="sv">{active_count}</div>
  </div>
  <div class="sc">
    <div class="sl">Administrateurs</div>
    <div class="sv">{admin_count}</div>
  </div>
  <div class="sc">
    <div class="sl">Employés</div>
    <div class="sv">{emp_count}</div>
  </div>
</div>

<div class="panel">
  <table>
    <thead><tr>
      <th>Identifiant</th><th>Nom complet</th><th>Rôle</th><th>Statut</th><th>Créé le</th><th>Actions</th>
    </tr></thead>
    <tbody>{rows}</tbody>
  </table>
</div>"#,
        n = users.len(),
    );

    Html(layout("Utilisateurs", "utilisateurs", &user, &state.tenant_name, &content))
}

pub async fn user_form(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Html<String> {
    let content = r#"<div class="page-title">Nouvel utilisateur</div>
<div class="page-sub">Créez un compte pour un employé ou un administrateur.</div>
<div class="panel">
  <div class="ph"><span class="pt">Informations du compte</span></div>
  <div style="padding:24px">
  <form method="post" action="/admin/utilisateurs/nouveau">
    <div class="form-grid">
      <div class="fg">
        <label for="username">Identifiant *</label>
        <input id="username" name="username" required placeholder="ex : jean.dupont" maxlength="100" autocomplete="off">
      </div>
      <div class="fg">
        <label for="full_name">Nom complet *</label>
        <input id="full_name" name="full_name" required placeholder="ex : Jean Dupont" maxlength="200">
      </div>
      <div class="fg">
        <label for="role">Rôle *</label>
        <select id="role" name="role">
          <option value="employee">Employé — accès stock uniquement</option>
          <option value="admin">Administrateur — accès complet</option>
        </select>
      </div>
      <div class="fg">
        <label for="password">Mot de passe *</label>
        <input id="password" name="password" type="password" required minlength="6" autocomplete="new-password" placeholder="Minimum 6 caractères">
      </div>
      <div class="fg">
        <label for="password2">Confirmer le mot de passe *</label>
        <input id="password2" name="password2" type="password" required autocomplete="new-password" placeholder="Répéter le mot de passe">
      </div>
    </div>
    <div class="fa">
      <button type="submit" class="btn btn-p">Créer le compte</button>
      <a href="/admin/utilisateurs" class="btn btn-g">Annuler</a>
    </div>
  </form>
  </div>
</div>"#;

    Html(layout("Nouvel utilisateur", "utilisateurs", &user, &state.tenant_name, content))
}

#[derive(Deserialize)]
pub struct UserForm {
    pub username: String,
    pub full_name: String,
    pub role: String,
    pub password: String,
    pub password2: String,
}

pub async fn user_create(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Form(f): Form<UserForm>,
) -> impl IntoResponse {
    let username = f.username.trim();
    let full_name = f.full_name.trim();

    if username.is_empty() || full_name.is_empty() {
        return error_admin(
            "Identifiant et nom complet sont obligatoires.",
            &user,
            &state.tenant_name,
        )
        .into_response();
    }
    if f.password.len() < 6 {
        return error_admin(
            "Le mot de passe doit contenir au moins 6 caractères.",
            &user,
            &state.tenant_name,
        )
        .into_response();
    }
    if f.password != f.password2 {
        return error_admin(
            "Les mots de passe ne correspondent pas.",
            &user,
            &state.tenant_name,
        )
        .into_response();
    }
    let role = if f.role == "admin" { "admin" } else { "employee" };

    match super::create_user(&state.pool, username, full_name, role, &f.password).await {
        Ok(_) => Redirect::to("/admin/utilisateurs").into_response(),
        Err(e) => {
            let msg = if e.to_string().contains("unique") {
                "Cet identifiant est déjà utilisé.".to_string()
            } else {
                e.to_string()
            };
            error_admin(&msg, &user, &state.tenant_name).into_response()
        }
    }
}

pub async fn user_toggle(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match super::toggle_user(&state.pool, id).await {
        Ok(_) => Redirect::to("/admin/utilisateurs").into_response(),
        Err(e) => error_admin(&e.to_string(), &user, &state.tenant_name).into_response(),
    }
}

pub async fn user_mdp_form(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
) -> Html<String> {
    let content = format!(
        r#"<div class="page-title">Changer le mot de passe</div>
<div class="page-sub">Définissez un nouveau mot de passe pour ce compte.</div>
<div class="panel">
  <div class="ph"><span class="pt">Nouveau mot de passe</span></div>
  <div style="padding:24px">
  <form method="post" action="/admin/utilisateurs/{id}/mdp">
    <div class="form-grid">
      <div class="fg">
        <label for="password">Nouveau mot de passe *</label>
        <input id="password" name="password" type="password" required minlength="6" autocomplete="new-password" placeholder="Minimum 6 caractères">
      </div>
      <div class="fg">
        <label for="password2">Confirmer *</label>
        <input id="password2" name="password2" type="password" required autocomplete="new-password" placeholder="Répéter le mot de passe">
      </div>
    </div>
    <div class="fa">
      <button type="submit" class="btn btn-p">Enregistrer</button>
      <a href="/admin/utilisateurs" class="btn btn-g">Annuler</a>
    </div>
  </form>
  </div>
</div>"#,
    );
    Html(layout("Changer le mot de passe", "utilisateurs", &user, &state.tenant_name, &content))
}

#[derive(Deserialize)]
pub struct MdpForm {
    pub password: String,
    pub password2: String,
}

pub async fn user_mdp_change(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
    Form(f): Form<MdpForm>,
) -> impl IntoResponse {
    if f.password.len() < 6 {
        return error_admin(
            "Le mot de passe doit contenir au moins 6 caractères.",
            &user,
            &state.tenant_name,
        )
        .into_response();
    }
    if f.password != f.password2 {
        return error_admin(
            "Les mots de passe ne correspondent pas.",
            &user,
            &state.tenant_name,
        )
        .into_response();
    }
    match super::update_password(&state.pool, id, &f.password).await {
        Ok(_) => Redirect::to("/admin/utilisateurs").into_response(),
        Err(e) => error_admin(&e.to_string(), &user, &state.tenant_name).into_response(),
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

fn error_admin(msg: &str, user: &User, tenant: &str) -> Html<String> {
    Html(layout(
        "Erreur",
        "utilisateurs",
        user,
        tenant,
        &format!(r#"<div class="err-bar">⚠️ <strong>Erreur :</strong> {}</div><a href="/admin/utilisateurs" class="btn btn-g">← Retour</a>"#, esc(msg)),
    ))
}
