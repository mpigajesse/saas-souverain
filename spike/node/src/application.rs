//! Hôte d'exécution (refonte V3, étape 3, ADR-007) : le nœud sert l'interface
//! de l'application externe telle que son éditeur l'a conçue, en relayant
//! toutes les requêtes qui ne visent pas sa propre console (`/amane/...`).
//!
//! L'application tourne dans son propre conteneur (service `amane-application`
//! de la pile) ; `AMANE_APP_URL` la désigne (ex. `http://amane-application:8080`)
//! et `AMANE_APP_SANTE` le chemin sondé pour connaître son état.

use std::time::Duration;

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use tokio::sync::watch;

/// Taille maximale d'un corps de requête relayé (téléversements compris).
const CORPS_MAX: usize = 64 * 1024 * 1024;

/// En-têtes propres à une connexion (RFC 9110 § 7.6.1) : jamais relayés.
const SAUT_A_SAUT: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

/// L'application hébergée.
pub struct Application {
    base: String,
    sante: String,
    client: reqwest::Client,
}

impl Application {
    /// Lit `AMANE_APP_URL` / `AMANE_APP_SANTE`. `None` si aucune application
    /// n'est hébergée sur ce cluster.
    pub fn depuis_env() -> Option<Self> {
        let base = std::env::var("AMANE_APP_URL").ok().filter(|v| !v.is_empty())?;
        let sante = std::env::var("AMANE_APP_SANTE").unwrap_or_else(|_| "/".into());
        Self::nouvelle(&base, &sante).ok()
    }

    pub fn nouvelle(base: &str, sante: &str) -> reqwest::Result<Self> {
        let client = reqwest::Client::builder()
            // Les redirections appartiennent au navigateur, pas au relais.
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            sante: if sante.starts_with('/') { sante.to_string() } else { format!("/{sante}") },
            client,
        })
    }

    /// URL cible pour un chemin (avec sa requête) reçu par le nœud.
    pub fn cible(&self, chemin_et_requete: &str) -> String {
        format!("{}{}", self.base, chemin_et_requete)
    }
}

/// Un en-tête est-il relayable ?
pub fn relayable(nom: &str) -> bool {
    let nom = nom.to_ascii_lowercase();
    !SAUT_A_SAUT.contains(&nom.as_str()) && nom != "host" && nom != "content-length"
}

/// Copie les en-têtes relayables.
fn copier_entetes(source: &HeaderMap, cible: &mut HeaderMap) {
    for (nom, valeur) in source {
        if relayable(nom.as_str()) {
            cible.append(nom.clone(), valeur.clone());
        }
    }
}

/// Relaie une requête vers l'application et renvoie sa réponse telle quelle
/// (statut, en-têtes, corps en flux).
pub async fn relayer(app: &Application, req: Request) -> Response {
    let (parties, corps) = req.into_parts();
    let chemin = parties.uri.path_and_query().map(|p| p.as_str()).unwrap_or("/");
    let octets = match axum::body::to_bytes(corps, CORPS_MAX).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::PAYLOAD_TOO_LARGE, "Requête trop volumineuse").into_response(),
    };

    let mut entetes = HeaderMap::new();
    copier_entetes(&parties.headers, &mut entetes);
    if let Some(hote) = parties.headers.get("host") {
        entetes.insert(HeaderName::from_static("x-forwarded-host"), hote.clone());
    }
    entetes.insert(HeaderName::from_static("x-forwarded-proto"), HeaderValue::from_static("http"));

    let reponse = app
        .client
        .request(parties.method.clone(), app.cible(chemin))
        .headers(entetes)
        .body(octets)
        .send()
        .await;
    let reponse = match reponse {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("L'application hébergée ne répond pas : {e}"),
            )
                .into_response()
        }
    };

    let mut sortie = Response::builder().status(reponse.status());
    if let Some(h) = sortie.headers_mut() {
        copier_entetes(reponse.headers(), h);
    }
    sortie
        .body(Body::from_stream(reponse.bytes_stream()))
        .unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response())
}

/// Sonde l'application toutes les 30 s et publie son état (`ok` / `ko`) pour
/// le battement de cœur vers le portail.
pub fn lancer_sonde(app: std::sync::Arc<Application>, canal: watch::Sender<String>) {
    tokio::spawn(async move {
        let mut dernier = String::new();
        loop {
            let url = app.cible(&app.sante);
            let etat = match app.client.get(&url).timeout(Duration::from_secs(5)).send().await {
                Ok(r) if r.status().is_success() || r.status().is_redirection() => "ok",
                _ => "ko",
            };
            if etat != dernier {
                println!("  Application : {} ({url})", if etat == "ok" { "disponible" } else { "INDISPONIBLE" });
                dernier = etat.to_string();
                let _ = canal.send_replace(dernier.clone());
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn en_tetes_de_connexion_jamais_relayes() {
        for nom in ["Connection", "keep-alive", "Transfer-Encoding", "Upgrade", "Host", "Content-Length"] {
            assert!(!relayable(nom), "{nom} ne doit pas être relayé");
        }
        for nom in ["Cookie", "Set-Cookie", "Content-Type", "Authorization", "Location"] {
            assert!(relayable(nom), "{nom} doit être relayé");
        }
    }

    #[test]
    fn cible_conserve_chemin_et_requete() {
        let app = Application::nouvelle("http://amane-application:8080/", "sante").unwrap();
        assert_eq!(app.cible("/factures?page=2"), "http://amane-application:8080/factures?page=2");
        assert_eq!(app.sante, "/sante");
    }

    #[tokio::test]
    async fn relaie_vers_l_application() {
        use axum::routing::post;
        // Fausse application : renvoie la méthode, le chemin et le corps reçus.
        let routeur = axum::Router::new().route(
            "/echo",
            post(|req: Request| async move {
                let chemin = req.uri().to_string();
                let corps = axum::body::to_bytes(req.into_body(), 1024).await.unwrap();
                (
                    [("set-cookie", "session=1"), ("connection", "close")],
                    format!("{chemin}|{}", String::from_utf8_lossy(&corps)),
                )
            }),
        );
        let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let adresse = ecoute.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(ecoute, routeur).await.unwrap() });

        let app = Application::nouvelle(&format!("http://{adresse}"), "/").unwrap();
        let req = Request::builder()
            .method("POST")
            .uri("/echo?x=1")
            .header("host", "poste-1:9001")
            .body(Body::from("bonjour"))
            .unwrap();
        let rep = relayer(&app, req).await;
        assert_eq!(rep.status(), StatusCode::OK);
        assert_eq!(rep.headers().get("set-cookie").unwrap(), "session=1");
        assert!(rep.headers().get("connection").is_none());
        let corps = axum::body::to_bytes(rep.into_body(), 1024).await.unwrap();
        assert_eq!(String::from_utf8_lossy(&corps), "/echo?x=1|bonjour");
    }

    #[tokio::test]
    async fn application_absente() {
        let app = Application::nouvelle("http://127.0.0.1:9", "/").unwrap();
        let rep = relayer(&app, Request::builder().uri("/").body(Body::empty()).unwrap()).await;
        assert_eq!(rep.status(), StatusCode::BAD_GATEWAY);
    }
}
