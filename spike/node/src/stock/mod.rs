use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub node_id: Uuid,
    pub tenant_name: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Article {
    pub id: Uuid,
    pub code: String,
    pub nom: String,
    pub description: Option<String>,
    pub categorie: Option<String>,
    pub unite: String,
    pub prix_unitaire: Option<f64>,
    pub seuil_alerte: i32,
    pub actif: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct StockItem {
    pub id: Uuid,
    pub code: String,
    pub nom: String,
    pub categorie: Option<String>,
    pub unite: String,
    pub seuil_alerte: i32,
    pub prix_unitaire: Option<f64>,
    pub stock_qty: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Mouvement {
    pub id: Uuid,
    pub article_code: String,
    pub article_nom: String,
    pub type_mvt: String,
    pub quantite: i32,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS articles (
            id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
            code          VARCHAR(50)  NOT NULL UNIQUE,
            nom           VARCHAR(200) NOT NULL,
            description   TEXT,
            categorie     VARCHAR(100),
            unite         VARCHAR(20)  NOT NULL DEFAULT 'unité',
            prix_unitaire FLOAT8,
            seuil_alerte  INTEGER      NOT NULL DEFAULT 0,
            actif         BOOLEAN      NOT NULL DEFAULT TRUE,
            created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
            updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS mouvements_stock (
            id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
            article_id  UUID        NOT NULL REFERENCES articles(id) ON DELETE RESTRICT,
            type_mvt    VARCHAR(20) NOT NULL CHECK (type_mvt IN ('entree','sortie','ajustement')),
            quantite    INTEGER     NOT NULL CHECK (quantite != 0),
            reference   VARCHAR(100),
            notes       TEXT,
            created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_stock_actuel(pool: &PgPool) -> Result<Vec<StockItem>> {
    Ok(sqlx::query_as::<_, StockItem>(
        "SELECT
             a.id, a.code, a.nom, a.categorie, a.unite,
             a.seuil_alerte, a.prix_unitaire,
             COALESCE(SUM(m.quantite), 0)::bigint AS stock_qty
         FROM articles a
         LEFT JOIN mouvements_stock m ON m.article_id = a.id
         WHERE a.actif = TRUE
         GROUP BY a.id, a.code, a.nom, a.categorie, a.unite, a.seuil_alerte, a.prix_unitaire
         ORDER BY a.nom",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_articles(pool: &PgPool) -> Result<Vec<Article>> {
    Ok(
        sqlx::query_as::<_, Article>("SELECT * FROM articles WHERE actif = TRUE ORDER BY nom")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn create_article(
    pool: &PgPool,
    code: &str,
    nom: &str,
    description: Option<&str>,
    categorie: Option<&str>,
    unite: &str,
    prix_unitaire: Option<f64>,
    seuil_alerte: i32,
) -> Result<Uuid> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO articles (code, nom, description, categorie, unite, prix_unitaire, seuil_alerte)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id",
    )
    .bind(code)
    .bind(nom)
    .bind(description)
    .bind(categorie)
    .bind(unite)
    .bind(prix_unitaire)
    .bind(seuil_alerte)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn delete_article(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query("UPDATE articles SET actif = FALSE, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_mouvements(pool: &PgPool, limit: i64) -> Result<Vec<Mouvement>> {
    Ok(sqlx::query_as::<_, Mouvement>(
        "SELECT
             m.id,
             a.code  AS article_code,
             a.nom   AS article_nom,
             m.type_mvt, m.quantite, m.reference, m.notes, m.created_at
         FROM mouvements_stock m
         JOIN articles a ON a.id = m.article_id
         ORDER BY m.created_at DESC
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn create_mouvement(
    pool: &PgPool,
    article_id: Uuid,
    type_mvt: &str,
    quantite: i32,
    reference: Option<&str>,
    notes: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO mouvements_stock (article_id, type_mvt, quantite, reference, notes)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(article_id)
    .bind(type_mvt)
    .bind(quantite)
    .bind(reference)
    .bind(notes)
    .execute(pool)
    .await?;
    Ok(())
}
