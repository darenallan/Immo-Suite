//! Quittance PDF, envois et liens de partage (étape 08 ; F10, DP-98, `API_CONTRACTS.md` §5.5, §6.1, §7.6), et page publique S26.
//!
//! - `GET /api/receipts/{id}/pdf` sert le fichier en flux par l'API (décision n° 68) ; `409 receipt.pdf_not_ready` sinon.
//! - `GET /s/{jeton}` : HTML rendu par le serveur, sans JavaScript ni style en ligne (CSP stricte), `noindex` ; le jeton
//!   n'apparaît dans aucun journal (le journal d'accès n'écrit que le modèle de route `/s/{token}`).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use immo_app::delivery::{self, PublicReceipt};
use immo_app::identity::authorize::Action;
use immo_app::identity::service::IdentityService;
use serde::Serialize;
use serde_json::{Map, Value, json};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::extract::{ApiJson, Ctx, Session, client_ip};
use crate::http::AppState;
use crate::problem::ApiError;

/// Débit de la page publique par adresse (DP-98) : 30 requêtes par minute.
const PUBLIC_LIMIT_PER_MINUTE: usize = 30;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/receipts/{id}/pdf", get(receipt_pdf))
        .route("/api/receipts/{id}/sends", get(list_sends).post(send_receipt))
        .route("/api/share-links/{id}/revoke", post(revoke_link))
}

/// Pages publiques S26 (hors `/api`, sans session).
pub fn public_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/s/{token}", get(public_page))
        .route("/s/{token}/pdf", get(public_pdf))
}

fn service(state: &AppState, ctx: &Ctx) -> Result<IdentityService, ApiError> {
    state.identity.clone().ok_or_else(|| ctx.unexpected())
}

fn parse_id(ctx: &Ctx, raw: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw.trim()).map_err(|_| ctx.err(ApiError::not_found()))
}

fn pdf_response(bytes: Vec<u8>, number: &str, inline: bool) -> Response {
    // nom de fichier généré depuis le numéro (format contraint), jamais depuis une saisie
    let safe: String = number.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
    let disposition = format!(
        "{}; filename=\"quittance-{safe}.pdf\"",
        if inline { "inline" } else { "attachment" }
    );
    let mut response = (StatusCode::OK, bytes).into_response();
    let h = response.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/pdf"));
    if let Ok(v) = HeaderValue::from_str(&disposition) {
        h.insert(header::CONTENT_DISPOSITION, v);
    }
    response
}

/// Corps de l'envoi (§6.1).
#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct SendPayload {
    /// `whatsapp`, `print_a4`, `print_a5` ou `link_copied`.
    pub channel: String,
}

/// Réponse d'un envoi : envoi consigné, lien (WhatsApp, lien copié), destinataire et message (WhatsApp).
#[derive(Debug, Serialize, ToSchema)]
pub struct SendBody {
    #[schema(value_type = Object)]
    pub send: Value,
    #[schema(value_type = Object)]
    pub share_link: Option<Value>,
    #[schema(value_type = Object)]
    pub recipient: Option<Value>,
    #[schema(value_type = Object)]
    pub message: Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SendsBody {
    #[schema(value_type = Vec<Object>)]
    pub items: Value,
}

/// PDF de la quittance en flux (DP-98, n° 68) ; `409 receipt.pdf_not_ready` avec `Retry-After` tant qu'il n'est pas rendu.
#[utoipa::path(get, path = "/api/receipts/{id}/pdf", params(("id" = String, Path), ("paper" = Option<String>, Query),
        ("disposition" = Option<String>, Query)),
    responses((status = 200, description = "PDF", content_type = "application/pdf"), (status = 409, description = "receipt.pdf_not_ready")))]
pub async fn receipt_pdf(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    Query(raw): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    session.require(Action::PaymentRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let paper = match raw.get("paper").map(String::as_str) {
        None | Some("a5") => "a5",
        Some("a4") => "a4",
        Some(_) => return Err(session.ctx.err(ApiError::validation_reason("paper"))),
    };
    let inline = raw.get("disposition").is_some_and(|d| d == "inline");
    let Some(store) = state.file_store.as_ref() else {
        return Err(session.ctx.unexpected());
    };
    match service(&state, &session.ctx)?
        .receipt_pdf(&session.auth, id, paper, store, &session.ctx.client)
        .await
    {
        Ok((bytes, number)) => Ok(pdf_response(bytes, &number, inline)),
        Err(e) => {
            let mut response = session.ctx.service(e).into_response();
            if response.status() == StatusCode::CONFLICT {
                response
                    .headers_mut()
                    .insert(header::RETRY_AFTER, HeaderValue::from_static("5"));
            }
            Ok(response)
        }
    }
}

/// Consigne un envoi (R-055) ; WhatsApp et lien copié créent un lien de partage (R-054) ; message minimal (SECURITY.md §7.8).
#[utoipa::path(post, path = "/api/receipts/{id}/sends", params(("id" = String, Path)), request_body = SendPayload,
    responses((status = 201, description = "Envoi consigné", body = SendBody), (status = 422, description = "receipt.recipient_has_no_phone")))]
pub async fn send_receipt(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    session.require(Action::ReceiptIssue)?;
    let id = parse_id(&session.ctx, &id)?;
    let ApiJson(map) = body?;
    if map.keys().any(|k| k != "channel") {
        return Err(session.ctx.err(ApiError::unknown_field()));
    }
    let channel = map
        .get("channel")
        .and_then(Value::as_str)
        .ok_or_else(|| session.ctx.err(ApiError::validation_reason("channel")))?
        .to_string();
    let v = service(&state, &session.ctx)?
        .send_receipt(&session.auth, id, &channel, &state.public_base_url, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((StatusCode::CREATED, Json(v)))
}

/// Historique des envois d'une quittance (R-055).
#[utoipa::path(get, path = "/api/receipts/{id}/sends", params(("id" = String, Path)),
    responses((status = 200, description = "Envois", body = SendsBody)))]
pub async fn list_sends(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    session.require(Action::PaymentRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let items = service(&state, &session.ctx)?
        .receipt_sends(&session.auth, id)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(json!({ "items": items })))
}

/// Révoque un lien de partage (R-054) : il répond ensuite 410 sur la page publique.
#[utoipa::path(post, path = "/api/share-links/{id}/revoke", params(("id" = String, Path)),
    responses((status = 200, description = "Lien révoqué", body = Object)))]
pub async fn revoke_link(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    session.require(Action::ReceiptIssue)?;
    let id = parse_id(&session.ctx, &id)?;
    let v = service(&state, &session.ctx)?
        .revoke_share_link(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(v))
}

// ───────── page publique S26 ─────────

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn page(status: StatusCode, title: &str, body: &str) -> Response {
    let html = format!(
        "<!doctype html><html lang=\"fr\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <meta name=\"robots\" content=\"noindex, nofollow\"><title>{}</title></head><body><main>{body}</main></body></html>",
        escape(title)
    );
    let mut response = (status, Html(html)).into_response();
    response
        .headers_mut()
        .insert("x-robots-tag", HeaderValue::from_static("noindex, nofollow"));
    response
}

fn limited(state: &AppState, request: &axum::extract::Request) -> bool {
    let peer = request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|c| c.0.ip());
    match client_ip(request.headers(), peer, state.trust_proxy) {
        Some(ip) => state
            .limiter
            .hit(&format!("pub:{ip}"), PUBLIC_LIMIT_PER_MINUTE, Duration::from_secs(60))
            .is_err(),
        None => false,
    }
}

fn receipt_html(token: &str, snapshot: &Value) -> String {
    let Some(v) = immo_pdf::receipt_view(snapshot) else {
        return "<p>Introuvable</p>".to_string();
    };
    let s = |k: &str| escape(v.get(k).and_then(Value::as_str).unwrap_or(""));
    let mut html = String::new();
    html.push_str(&format!("<p>{}</p>", s("issuer_name")));
    if let Some(w) = v.get("watermark").and_then(Value::as_str) {
        html.push_str(&format!("<p><strong>{}</strong></p>", escape(w)));
    }
    html.push_str(&format!(
        "<h1>Quittance de loyer n° {}</h1><p>Émise le {}</p>",
        s("number"),
        s("issued_on")
    ));
    html.push_str(&format!("<p>Reçu de : {}</p>", s("tenants")));
    if let Some(p) = v.get("paid_by").and_then(Value::as_str) {
        html.push_str(&format!("<p>{}</p>", escape(p)));
    }
    html.push_str(&format!("<p>Pour le lot : {}</p>", s("unit_line")));
    html.push_str(&format!(
        "<p>{}</p><p>Date du paiement : {} · {}</p><ul>",
        s("period_line"),
        s("paid_on"),
        s("mode")
    ));
    for line in v.get("lines").and_then(Value::as_array).cloned().unwrap_or_default() {
        let label = line.get(0).and_then(Value::as_str).unwrap_or("");
        let amount = line.get(1).and_then(Value::as_str).unwrap_or("");
        html.push_str(&format!("<li>{} : {}</li>", escape(label), escape(amount)));
    }
    html.push_str(&format!(
        "</ul><p><strong>Total reçu : {}</strong></p><p>{}</p><p>{}</p>",
        s("total"),
        s("words"),
        s("remaining_line")
    ));
    if let Some(a) = v.get("advance_line").and_then(Value::as_str) {
        html.push_str(&format!("<p>{}</p>", escape(a)));
    }
    html.push_str(&format!(
        "<p><a href=\"/s/{token}/pdf\">Télécharger le PDF</a> · <a href=\"/s/{token}/pdf?paper=a4\">Format A4</a></p>"
    ));
    html
}

/// S26 : la quittance seule ; 410 si le lien a expiré ou a été révoqué (E-12) ; 404 si le jeton est inconnu.
pub async fn public_page(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
    request: axum::extract::Request,
) -> Response {
    if limited(&state, &request) {
        return page(
            StatusCode::TOO_MANY_REQUESTS,
            "Trop de demandes",
            "<p>Trop de demandes. Réessayez dans une minute.</p>",
        );
    }
    let Some(identity) = state.identity.as_ref() else {
        return page(StatusCode::NOT_FOUND, "Introuvable", "<p>Introuvable</p>");
    };
    match delivery::resolve_share_link(identity.db_handle(), identity.clock_handle(), &token).await {
        Ok(PublicReceipt::Found { snapshot, .. }) => {
            page(StatusCode::OK, "Quittance", &receipt_html(&token, &snapshot))
        }
        Ok(PublicReceipt::Expired { organization }) => page(
            StatusCode::GONE,
            "Lien expiré",
            &format!("<p>Ce lien a expiré. Demandez un nouveau lien à {}.</p>", escape(&organization)),
        ),
        Ok(PublicReceipt::Unknown) => page(StatusCode::NOT_FOUND, "Introuvable", "<p>Introuvable</p>"),
        Err(e) => {
            tracing::error!(error = %e, "page publique : erreur de base");
            page(
                StatusCode::SERVICE_UNAVAILABLE,
                "Indisponible",
                "<p>Service indisponible. Réessayez.</p>",
            )
        }
    }
}

/// PDF d'une quittance partagée (même contrôle que la page).
pub async fn public_pdf(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
    Query(raw): Query<HashMap<String, String>>,
    request: axum::extract::Request,
) -> Response {
    if limited(&state, &request) {
        return page(
            StatusCode::TOO_MANY_REQUESTS,
            "Trop de demandes",
            "<p>Trop de demandes. Réessayez dans une minute.</p>",
        );
    }
    let (Some(identity), Some(store)) = (state.identity.as_ref(), state.file_store.as_ref()) else {
        return page(StatusCode::NOT_FOUND, "Introuvable", "<p>Introuvable</p>");
    };
    let db = identity.db_handle();
    match delivery::resolve_share_link(db, identity.clock_handle(), &token).await {
        Ok(PublicReceipt::Found { org, receipt, snapshot, .. }) => {
            match delivery::public_pdf(db, store, org, receipt, raw.get("paper").map(String::as_str)).await {
                Ok(Some(bytes)) => {
                    let number = snapshot
                        .pointer("/receipt/number")
                        .and_then(Value::as_str)
                        .unwrap_or("quittance")
                        .to_string();
                    pdf_response(bytes, &number, false)
                }
                _ => page(
                    StatusCode::CONFLICT,
                    "PDF en préparation",
                    "<p>Le PDF est en préparation. Réessayez dans un instant.</p>",
                ),
            }
        }
        Ok(PublicReceipt::Expired { organization }) => page(
            StatusCode::GONE,
            "Lien expiré",
            &format!("<p>Ce lien a expiré. Demandez un nouveau lien à {}.</p>", escape(&organization)),
        ),
        Ok(PublicReceipt::Unknown) => page(StatusCode::NOT_FOUND, "Introuvable", "<p>Introuvable</p>"),
        Err(e) => {
            tracing::error!(error = %e, "page publique : erreur de base");
            page(
                StatusCode::SERVICE_UNAVAILABLE,
                "Indisponible",
                "<p>Service indisponible. Réessayez.</p>",
            )
        }
    }
}
