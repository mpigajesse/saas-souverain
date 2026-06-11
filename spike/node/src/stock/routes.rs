use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{AppState, User};
use super::{Mouvement, StockItem};

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

fn layout(title: &str, active: &str, user: &User, state: &AppState, content: &str) -> String {
    crate::auth::routes::layout(title, active, user, &state.tenant_name, content)
}

fn err(user: &User, state: &AppState, msg: &str) -> Html<String> {
    Html(layout(
        "Erreur",
        "",
        user,
        state,
        &format!(
            r#"<div class="err-bar">⚠️ <strong>Erreur :</strong> {}</div>"#,
            esc(msg)
        ),
    ))
}

// ── Dashboard ────────────────────────────────────────────────────────────────

pub async fn dashboard(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Html<String> {
    let (items, mvts) = tokio::join!(
        super::get_stock_actuel(&state.pool),
        super::get_mouvements(&state.pool, 8),
    );
    let items = match items {
        Ok(v) => v,
        Err(e) => return err(&user, &state, &e.to_string()),
    };
    let mvts = match mvts {
        Ok(v) => v,
        Err(e) => return err(&user, &state, &e.to_string()),
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

    Html(layout("Tableau de bord", "dashboard", &user, &state, &content))
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

pub async fn articles_list(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Html<String> {
    let (items, articles) = tokio::join!(
        super::get_stock_actuel(&state.pool),
        super::get_articles(&state.pool),
    );
    let items = match items {
        Ok(v) => v,
        Err(e) => return err(&user, &state, &e.to_string()),
    };
    let articles = match articles {
        Ok(v) => v,
        Err(e) => return err(&user, &state, &e.to_string()),
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
    Html(layout("Articles", "articles", &user, &state, &content))
}

pub async fn article_form(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Html<String> {
    Html(layout("Nouvel article", "articles", &user, &state, ARTICLE_FORM))
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
    Extension(user): Extension<User>,
    Form(f): Form<ArticleForm>,
) -> impl IntoResponse {
    let code = f.code.trim().to_string();
    let nom = f.nom.trim().to_string();
    if code.is_empty() || nom.is_empty() {
        return err(&user, &state, "Code et désignation sont obligatoires.").into_response();
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
        Err(e) => err(&user, &state, &e.to_string()).into_response(),
    }
}

pub async fn article_delete(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match super::delete_article(&state.pool, id).await {
        Ok(_) => Redirect::to("/articles").into_response(),
        Err(e) => err(&user, &state, &e.to_string()).into_response(),
    }
}

// ── Mouvements ───────────────────────────────────────────────────────────────

pub async fn mouvements_page(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<User>,
) -> Html<String> {
    let (articles, mvts) = tokio::join!(
        super::get_articles(&state.pool),
        super::get_mouvements(&state.pool, 50),
    );
    let articles = match articles {
        Ok(v) => v,
        Err(e) => return err(&user, &state, &e.to_string()),
    };
    let mvts = match mvts {
        Ok(v) => v,
        Err(e) => return err(&user, &state, &e.to_string()),
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

    Html(layout("Mouvements", "mouvements", &user, &state, &content))
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
    Extension(user): Extension<User>,
    Form(f): Form<MouvementForm>,
) -> impl IntoResponse {
    let article_id = match f.article_id.trim().parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return err(&user, &state, "Article invalide.").into_response(),
    };
    let mut qty: i32 = match f.quantite.trim().parse() {
        Ok(q) if q != 0 => q,
        _ => return err(&user, &state, "La quantité doit être un entier non nul.").into_response(),
    };
    let type_mvt = f.type_mvt.trim();
    if !["entree", "sortie", "ajustement"].contains(&type_mvt) {
        return err(&user, &state, "Type de mouvement invalide.").into_response();
    }
    if type_mvt == "sortie" && qty > 0 {
        qty = -qty;
    }
    let reference = f.reference.as_deref().filter(|s| !s.trim().is_empty());
    let notes = f.notes.as_deref().filter(|s| !s.trim().is_empty());

    match super::create_mouvement(&state.pool, article_id, type_mvt, qty, reference, notes).await {
        Ok(_) => Redirect::to("/mouvements").into_response(),
        Err(e) => err(&user, &state, &e.to_string()).into_response(),
    }
}
