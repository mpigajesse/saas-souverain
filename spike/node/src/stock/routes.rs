use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{AppState, Mouvement, StockItem};

// ── HTML escaping ────────────────────────────────────────────────────────────

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn esc_opt(s: &Option<String>) -> String {
    s.as_deref().map(esc).unwrap_or_default()
}

// ── CSS ──────────────────────────────────────────────────────────────────────

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
.sb-node{font-size:.72rem;color:rgba(255,255,255,.3);line-height:1.6}
.sb-node strong{color:rgba(255,255,255,.55)}
.sb-dot{display:inline-block;width:6px;height:6px;background:#22C55E;border-radius:50%;margin-right:5px;animation:pulse 2s infinite}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:.3}}

/* ── Main ── */
.main{margin-left:240px;flex:1;min-height:100vh;display:flex;flex-direction:column}
.topbar{background:var(--card);border-bottom:1px solid var(--bd);padding:0 32px;height:56px;display:flex;align-items:center;justify-content:space-between;position:sticky;top:0;z-index:100}
.topbar-title{font-size:1rem;font-weight:700}
.topbar-right{display:flex;align-items:center;gap:10px}
.container{padding:28px 32px;flex:1}

/* ── Typography ── */
.page-title{font-size:1.35rem;font-weight:800;margin-bottom:4px}
.page-sub{color:var(--mu);font-size:.875rem;margin-bottom:24px}
.row-split{display:flex;align-items:center;justify-content:space-between;margin-bottom:24px}

/* ── Stats ── */
.stats{display:grid;grid-template-columns:repeat(3,1fr);gap:16px;margin-bottom:24px}
.sc{background:var(--card);border:1px solid var(--bd);border-radius:12px;padding:18px 22px}
.sl{font-size:.7rem;font-weight:700;text-transform:uppercase;letter-spacing:.07em;color:var(--mu);margin-bottom:8px}
.sv{font-size:1.9rem;font-weight:800}
.sv.danger{color:var(--cr)}

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
.entree{background:#D1FAE5;color:#065F46}
.sortie{background:#FEE2E2;color:#991B1B}
.ajust{background:#FEF3C7;color:#92400E}

/* ── Buttons ── */
.btn{display:inline-flex;align-items:center;gap:6px;border:none;border-radius:8px;padding:8px 16px;font-size:.85rem;font-weight:600;cursor:pointer;text-decoration:none;transition:all .15s;font-family:inherit}
.btn-p{background:var(--cr);color:#fff}
.btn-p:hover{background:#8B1119;color:#fff}
.btn-sm{padding:4px 10px;font-size:.76rem}
.btn-g{background:transparent;border:1px solid var(--bd);color:var(--mu)}
.btn-g:hover{border-color:var(--tx);color:var(--tx)}
.btn-d{background:#FEE2E2;color:#991B1B;border:1px solid #FECACA}
.btn-d:hover{background:#FECACA}

/* ── Forms ── */
.form-grid{display:grid;grid-template-columns:1fr 1fr;gap:16px}
.fg{display:flex;flex-direction:column;gap:6px}
.full{grid-column:1/-1}
label{font-size:.81rem;font-weight:600}
input,select,textarea{border:1px solid var(--bd);border-radius:8px;padding:8px 12px;font-size:.875rem;font-family:inherit;width:100%;background:#fff;transition:border-color .15s}
input:focus,select:focus,textarea:focus{outline:none;border-color:var(--cr);box-shadow:0 0 0 3px rgba(160,21,32,.1)}
textarea{min-height:72px;resize:vertical}
.fa{display:flex;gap:10px;margin-top:8px}

/* ── Misc ── */
.empty{text-align:center;padding:36px 24px;color:var(--mu);font-size:.875rem}
.alert-bar{background:#FEF3C7;border:1px solid #FCD34D;border-radius:10px;padding:11px 16px;font-size:.84rem;color:#92400E;margin-bottom:18px;display:flex;align-items:center;gap:8px}

/* ── Responsive ── */
@media(max-width:900px){
  .sidebar{width:200px}
  .main{margin-left:200px}
  .container{padding:20px 18px}
}
@media(max-width:640px){
  .sidebar{display:none}
  .main{margin-left:0}
  .stats{grid-template-columns:1fr}
  .form-grid{grid-template-columns:1fr}
}
"#;

// ── Layout ───────────────────────────────────────────────────────────────────

fn layout(title: &str, active: &str, tenant: &str, content: &str) -> String {
    let da = if active == "dashboard" { "active" } else { "" };
    let aa = if active == "articles" { "active" } else { "" };
    let ma = if active == "mouvements" { "active" } else { "" };
    let tenant = esc(tenant);
    let initial = tenant.chars().next().unwrap_or('P').to_uppercase().to_string();
    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>{title} — {tenant}</title>
<style>{CSS}</style>
</head>
<body>

<!-- ── Sidebar ── -->
<aside class="sidebar">
  <div class="sb-brand">
    <div class="sb-sq">{initial}</div>
    <div>
      <div class="sb-name">{tenant}</div>
      <div class="sb-sub">Gestion PME</div>
    </div>
  </div>

  <div class="sb-section">Navigation</div>
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
        <path d="M21 8V21H3V8"/><path d="M23 3H1v5h22V3z"/>
        <path d="M10 12h4"/>
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

  <div class="sb-footer">
    <div class="sb-node">
      <span class="sb-dot"></span><strong>Nœud actif</strong><br>
      Données sécurisées sur site<br>
      XChaCha20-Poly1305
    </div>
  </div>
</aside>

<!-- ── Main ── -->
<div class="main">
  <div class="topbar">
    <span class="topbar-title">{title}</span>
    <div class="topbar-right">
      <span style="font-size:.78rem;color:var(--mu)">{tenant}</span>
    </div>
  </div>
  <div class="container">
{content}
  </div>
</div>

</body>
</html>"#
    )
}

fn error_html(tenant: &str, msg: &str) -> Html<String> {
    Html(layout(
        "Erreur",
        "",
        tenant,
        &format!(
            r#"<div class="alert-bar">⚠️ <strong>Erreur :</strong> {}</div>"#,
            esc(msg)
        ),
    ))
}

// ── Dashboard ────────────────────────────────────────────────────────────────

pub async fn dashboard(State(state): State<Arc<AppState>>) -> Html<String> {
    let (items, mvts) = tokio::join!(
        super::get_stock_actuel(&state.pool),
        super::get_mouvements(&state.pool, 8),
    );
    let items = match items {
        Ok(v) => v,
        Err(e) => return error_html(&state.tenant_name, &e.to_string()),
    };
    let mvts = match mvts {
        Ok(v) => v,
        Err(e) => return error_html(&state.tenant_name, &e.to_string()),
    };

    let total = items.len();
    let en_alerte = items
        .iter()
        .filter(|i| i.seuil_alerte > 0 && i.stock_qty <= i.seuil_alerte as i64)
        .count();

    let alerte_bar = if en_alerte > 0 {
        format!(
            r#"<div class="alert-bar">⚠️ <strong>{} article(s)</strong> en dessous du seuil d'alerte.</div>"#,
            en_alerte
        )
    } else {
        String::new()
    };

    let stat_cards = format!(
        r#"<div class="stats">
  <div class="sc"><div class="sl">Articles actifs</div><div class="sv">{total}</div></div>
  <div class="sc"><div class="sl">En alerte stock</div><div class="sv{dc}">{en_alerte}</div></div>
  <div class="sc"><div class="sl">Derniers mouvements</div><div class="sv">{nm}</div></div>
</div>"#,
        dc = if en_alerte > 0 { " danger" } else { "" },
        nm = mvts.len(),
    );

    let stock_rows: String = if items.is_empty() {
        r#"<tr><td colspan="6" class="empty">Aucun article — <a href="/articles/nouveau">créer le premier</a></td></tr>"#.into()
    } else {
        items.iter().map(|i| stock_row(i)).collect()
    };

    let mvt_rows: String = if mvts.is_empty() {
        r#"<tr><td colspan="5" class="empty">Aucun mouvement</td></tr>"#.into()
    } else {
        mvts.iter().map(|m| mvt_row_small(m)).collect()
    };

    let content = format!(
        r#"{alerte_bar}
{stat_cards}
<div class="panel">
  <div class="ph">
    <span class="pt">Stock actuel</span>
    <a href="/mouvements" class="btn btn-p btn-sm">+ Saisir mouvement</a>
  </div>
  <table>
    <thead><tr>
      <th>Code</th><th>Article</th><th>Catégorie</th>
      <th>Quantité</th><th>Seuil</th><th>Prix unitaire</th>
    </tr></thead>
    <tbody>{stock_rows}</tbody>
  </table>
</div>
<div class="panel">
  <div class="ph"><span class="pt">Derniers mouvements</span><a href="/mouvements" class="btn btn-g btn-sm">Voir tout</a></div>
  <table>
    <thead><tr><th>Date</th><th>Article</th><th>Type</th><th>Quantité</th><th>Référence</th></tr></thead>
    <tbody>{mvt_rows}</tbody>
  </table>
</div>"#
    );

    Html(layout("Tableau de bord", "dashboard", &state.tenant_name, &content))
}

fn stock_row(i: &StockItem) -> String {
    let en_alerte = i.seuil_alerte > 0 && i.stock_qty <= i.seuil_alerte as i64;
    let badge = if en_alerte {
        r#"<span class="badge alerte">Alerte</span>"#
    } else {
        r#"<span class="badge ok">OK</span>"#
    };
    let prix = i
        .prix_unitaire
        .map(|p| format!("{:.2} DH", p))
        .unwrap_or_else(|| "—".into());
    let qty_style = if en_alerte { " style=\"color:#A01520;font-weight:700\"" } else { "" };
    format!(
        r#"<tr>
  <td class="mono">{code}</td>
  <td><strong>{nom}</strong></td>
  <td>{cat}</td>
  <td{qty_style}>{qty} {unite} {badge}</td>
  <td>{seuil}</td>
  <td>{prix}</td>
</tr>"#,
        code = esc(&i.code),
        nom = esc(&i.nom),
        cat = esc_opt(&i.categorie),
        qty = i.stock_qty,
        unite = esc(&i.unite),
        seuil = i.seuil_alerte,
    )
}

fn mvt_row_small(m: &Mouvement) -> String {
    let (badge_class, label, sign) = match m.type_mvt.as_str() {
        "entree" => ("entree", "Entrée", "+"),
        "sortie" => ("sortie", "Sortie", ""),
        _ => ("ajust", "Ajust.", if m.quantite >= 0 { "+" } else { "" }),
    };
    format!(
        r#"<tr>
  <td class="mono" style="color:var(--mu)">{date}</td>
  <td>{code} — {nom}</td>
  <td><span class="badge {badge_class}">{label}</span></td>
  <td style="font-weight:700">{sign}{qty}</td>
  <td style="color:var(--mu)">{ref}</td>
</tr>"#,
        date = m.created_at.format("%d/%m %H:%M"),
        code = esc(&m.article_code),
        nom = esc(&m.article_nom),
        qty = m.quantite.abs(),
        ref = esc_opt(&m.reference),
    )
}

// ── Articles ─────────────────────────────────────────────────────────────────

pub async fn articles_list(State(state): State<Arc<AppState>>) -> Html<String> {
    let (items, articles) = tokio::join!(
        super::get_stock_actuel(&state.pool),
        super::get_articles(&state.pool),
    );
    let items = match items {
        Ok(v) => v,
        Err(e) => return error_html(&state.tenant_name, &e.to_string()),
    };
    let articles = match articles {
        Ok(v) => v,
        Err(e) => return error_html(&state.tenant_name, &e.to_string()),
    };

    let stock_map: std::collections::HashMap<Uuid, i64> =
        items.iter().map(|i| (i.id, i.stock_qty)).collect();

    let rows: String = if articles.is_empty() {
        r#"<tr><td colspan="7" class="empty">Aucun article — créez-en un ci-dessous.</td></tr>"#.into()
    } else {
        articles
            .iter()
            .map(|a| {
                let qty = stock_map.get(&a.id).copied().unwrap_or(0);
                let prix = a.prix_unitaire.map(|p| format!("{:.2} DH", p)).unwrap_or_else(|| "—".into());
                format!(
                    r#"<tr>
  <td class="mono">{code}</td>
  <td><strong>{nom}</strong></td>
  <td>{cat}</td>
  <td>{unite}</td>
  <td style="font-weight:600">{qty}</td>
  <td>{seuil}</td>
  <td>{prix}</td>
  <td>
    <form method="post" action="/articles/{id}/supprimer" style="display:inline"
          onsubmit="return confirm('Archiver cet article ?')">
      <button type="submit" class="btn btn-d btn-sm">Archiver</button>
    </form>
  </td>
</tr>"#,
                    code = esc(&a.code),
                    nom = esc(&a.nom),
                    cat = esc_opt(&a.categorie),
                    unite = esc(&a.unite),
                    seuil = a.seuil_alerte,
                    id = a.id,
                )
            })
            .collect()
    };

    let content = format!(
        r#"<div class="row-split">
  <div><div class="page-title">Articles</div><div class="page-sub">{n} article(s) actif(s)</div></div>
  <a href="/articles/nouveau" class="btn btn-p">+ Nouvel article</a>
</div>
<div class="panel">
  <table>
    <thead><tr>
      <th>Code</th><th>Article</th><th>Catégorie</th><th>Unité</th>
      <th>Qté stock</th><th>Seuil</th><th>Prix</th><th>Action</th>
    </tr></thead>
    <tbody>{rows}</tbody>
  </table>
</div>"#,
        n = articles.len(),
    );
    Html(layout("Articles", "articles", &state.tenant_name, &content))
}

pub async fn article_form(State(state): State<Arc<AppState>>) -> Html<String> {
    Html(layout("Nouvel article", "articles", &state.tenant_name, ARTICLE_FORM))
}

const ARTICLE_FORM: &str = r#"
<div class="page-title">Nouvel article</div>
<div class="page-sub">Remplissez les informations de l'article.</div>
<div class="panel">
  <div class="ph"><span class="pt">Informations article</span></div>
  <div style="padding:24px">
  <form method="post" action="/articles/nouveau">
    <div class="form-grid">
      <div class="fg">
        <label for="code">Code article *</label>
        <input id="code" name="code" required placeholder="ART-001" maxlength="50">
      </div>
      <div class="fg">
        <label for="nom">Désignation *</label>
        <input id="nom" name="nom" required placeholder="Nom du produit" maxlength="200">
      </div>
      <div class="fg">
        <label for="categorie">Catégorie</label>
        <input id="categorie" name="categorie" placeholder="ex : Matières premières" maxlength="100">
      </div>
      <div class="fg">
        <label for="unite">Unité de mesure *</label>
        <select id="unite" name="unite">
          <option value="unité">Unité (pcs)</option>
          <option value="kg">Kilogramme (kg)</option>
          <option value="g">Gramme (g)</option>
          <option value="L">Litre (L)</option>
          <option value="m">Mètre (m)</option>
          <option value="m²">Mètre carré (m²)</option>
          <option value="boîte">Boîte</option>
          <option value="palette">Palette</option>
        </select>
      </div>
      <div class="fg">
        <label for="prix_unitaire">Prix unitaire (DH)</label>
        <input id="prix_unitaire" name="prix_unitaire" type="number" step="0.01" min="0" placeholder="0.00">
      </div>
      <div class="fg">
        <label for="seuil_alerte">Seuil d'alerte (qté min)</label>
        <input id="seuil_alerte" name="seuil_alerte" type="number" min="0" value="0" placeholder="0">
      </div>
      <div class="fg full">
        <label for="description">Description (optionnel)</label>
        <textarea id="description" name="description" placeholder="Détails, référence fournisseur…"></textarea>
      </div>
    </div>
    <div class="fa">
      <button type="submit" class="btn btn-p">Créer l'article</button>
      <a href="/articles" class="btn btn-g">Annuler</a>
    </div>
  </form>
  </div>
</div>
"#;

#[derive(Deserialize)]
pub struct ArticleForm {
    pub code: String,
    pub nom: String,
    pub description: Option<String>,
    pub categorie: Option<String>,
    pub unite: String,
    pub prix_unitaire: Option<String>,
    pub seuil_alerte: Option<String>,
}

pub async fn article_create(
    State(state): State<Arc<AppState>>,
    Form(f): Form<ArticleForm>,
) -> impl IntoResponse {
    let code = f.code.trim().to_string();
    let nom = f.nom.trim().to_string();
    if code.is_empty() || nom.is_empty() {
        return error_html(&state.tenant_name, "Code et désignation sont obligatoires.").into_response();
    }
    let prix = f
        .prix_unitaire
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| s.trim().parse::<f64>().ok());
    let seuil = f
        .seuil_alerte
        .as_deref()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let desc = f.description.as_deref().filter(|s| !s.trim().is_empty());
    let cat = f.categorie.as_deref().filter(|s| !s.trim().is_empty());

    match super::create_article(&state.pool, &code, &nom, desc, cat, &f.unite, prix, seuil).await {
        Ok(_) => Redirect::to("/articles").into_response(),
        Err(e) => error_html(&state.tenant_name, &e.to_string()).into_response(),
    }
}

pub async fn article_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match super::delete_article(&state.pool, id).await {
        Ok(_) => Redirect::to("/articles").into_response(),
        Err(e) => error_html(&state.tenant_name, &e.to_string()).into_response(),
    }
}

// ── Mouvements ───────────────────────────────────────────────────────────────

pub async fn mouvements_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let (articles, mvts) = tokio::join!(
        super::get_articles(&state.pool),
        super::get_mouvements(&state.pool, 50),
    );
    let articles = match articles {
        Ok(v) => v,
        Err(e) => return error_html(&state.tenant_name, &e.to_string()),
    };
    let mvts = match mvts {
        Ok(v) => v,
        Err(e) => return error_html(&state.tenant_name, &e.to_string()),
    };

    let options: String = articles
        .iter()
        .map(|a| {
            format!(
                r#"<option value="{id}">[{code}] {nom}</option>"#,
                id = a.id,
                code = esc(&a.code),
                nom = esc(&a.nom),
            )
        })
        .collect();

    let no_articles = if articles.is_empty() {
        r#"<div class="alert-bar">⚠️ Aucun article — <a href="/articles/nouveau">créez-en un d'abord</a>.</div>"#
    } else {
        ""
    };

    let rows: String = if mvts.is_empty() {
        r#"<tr><td colspan="6" class="empty">Aucun mouvement enregistré.</td></tr>"#.into()
    } else {
        mvts.iter().map(|m| mvt_row_full(m)).collect()
    };

    let content = format!(
        r#"<div class="page-title">Mouvements de stock</div>
<div class="page-sub">Enregistrez les entrées, sorties et ajustements.</div>
{no_articles}
<div class="panel" style="margin-bottom:24px">
  <div class="ph"><span class="pt">Saisir un mouvement</span></div>
  <div style="padding:24px">
  <form method="post" action="/mouvements">
    <div class="form-grid">
      <div class="fg">
        <label for="article_id">Article *</label>
        <select id="article_id" name="article_id" required>
          <option value="">— Sélectionner —</option>
          {options}
        </select>
      </div>
      <div class="fg">
        <label for="type_mvt">Type *</label>
        <select id="type_mvt" name="type_mvt" required>
          <option value="entree">Entrée (réception, achat)</option>
          <option value="sortie">Sortie (vente, consommation)</option>
          <option value="ajustement">Ajustement inventaire</option>
        </select>
      </div>
      <div class="fg">
        <label for="quantite">Quantité *
          <span style="color:var(--mu);font-weight:400">(négatif pour réduire en ajustement)</span>
        </label>
        <input id="quantite" name="quantite" type="number" required
               placeholder="ex : 10" min="-999999" max="999999">
      </div>
      <div class="fg">
        <label for="reference">Référence (BL, BC, facture…)</label>
        <input id="reference" name="reference" placeholder="ex : BL-2024-001" maxlength="100">
      </div>
      <div class="fg full">
        <label for="notes">Notes</label>
        <textarea id="notes" name="notes" placeholder="Observations…" rows="2"></textarea>
      </div>
    </div>
    <div class="fa">
      <button type="submit" class="btn btn-p">Enregistrer le mouvement</button>
    </div>
  </form>
  </div>
</div>
<div class="panel">
  <div class="ph"><span class="pt">Historique des mouvements</span></div>
  <table>
    <thead><tr>
      <th>Date</th><th>Code</th><th>Article</th><th>Type</th><th>Quantité</th><th>Référence</th>
    </tr></thead>
    <tbody>{rows}</tbody>
  </table>
</div>"#
    );

    Html(layout("Mouvements", "mouvements", &state.tenant_name, &content))
}

fn mvt_row_full(m: &Mouvement) -> String {
    let (badge_class, label, sign) = match m.type_mvt.as_str() {
        "entree" => ("entree", "Entrée", "+"),
        "sortie" => ("sortie", "Sortie", "−"),
        _ => ("ajust", "Ajust.", if m.quantite >= 0 { "+" } else { "−" }),
    };
    let qty_color = match m.type_mvt.as_str() {
        "entree" => "color:#065F46",
        "sortie" => "color:#991B1B",
        _ => "",
    };
    format!(
        r#"<tr>
  <td class="mono" style="color:var(--mu)">{date}</td>
  <td class="mono">{code}</td>
  <td>{nom}</td>
  <td><span class="badge {badge_class}">{label}</span></td>
  <td style="font-weight:700;{qty_color}">{sign}{qty}</td>
  <td style="color:var(--mu)">{ref}</td>
</tr>"#,
        date = m.created_at.format("%d/%m/%Y %H:%M"),
        code = esc(&m.article_code),
        nom = esc(&m.article_nom),
        qty = m.quantite.abs(),
        ref = esc_opt(&m.reference),
    )
}

#[derive(Deserialize)]
pub struct MouvementForm {
    pub article_id: String,
    pub type_mvt: String,
    pub quantite: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

pub async fn mouvement_create(
    State(state): State<Arc<AppState>>,
    Form(f): Form<MouvementForm>,
) -> impl IntoResponse {
    let article_id = match f.article_id.trim().parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return error_html(&state.tenant_name, "Article invalide.").into_response(),
    };
    let mut qty: i32 = match f.quantite.trim().parse() {
        Ok(q) if q != 0 => q,
        _ => return error_html(&state.tenant_name, "La quantité doit être un entier non nul.").into_response(),
    };
    let type_mvt = f.type_mvt.trim();
    if !["entree", "sortie", "ajustement"].contains(&type_mvt) {
        return error_html(&state.tenant_name, "Type de mouvement invalide.").into_response();
    }
    // Pour une sortie, on stocke la quantité en négatif
    if type_mvt == "sortie" && qty > 0 {
        qty = -qty;
    }
    let reference = f.reference.as_deref().filter(|s| !s.trim().is_empty());
    let notes = f.notes.as_deref().filter(|s| !s.trim().is_empty());

    match super::create_mouvement(&state.pool, article_id, type_mvt, qty, reference, notes).await {
        Ok(_) => Redirect::to("/mouvements").into_response(),
        Err(e) => error_html(&state.tenant_name, &e.to_string()).into_response(),
    }
}
