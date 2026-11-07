//! Routes du patrimoine (étape 05 ; F04, F05, F06, DP-92) : biens, lots, locataires, fiches mandants, recherche unique.
//!
//! Chaque gestionnaire vérifie d'abord le droit (`Session::require`), puis lit le corps : un rôle sans droit reçoit `403` avant
//! toute erreur de saisie (DP-89). Les corps sont des objets dont les clés sont contrôlées ici (`400 request.unknown_field`) et
//! les valeurs par le service (`422`, une erreur par champ).

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header::ETAG};
use axum::routing::{get, post};
use axum::{Json, Router};
use immo_app::catalog::{Input, ListQuery, OwnerView, Page, PropertyView, SearchResult, TenantView, UnitView};
use immo_app::identity::authorize::Action;
use immo_app::identity::service::IdentityService;
use immo_app::identity::settings::rfc3339;
use serde::Serialize;
use serde_json::{Map, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::extract::{ApiJson, Ctx, Session};
use crate::http::AppState;
use crate::problem::ApiError;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/properties", get(list_properties).post(create_property))
        .route("/api/properties/{id}", get(get_property).patch(patch_property))
        .route("/api/properties/{id}/archive", post(archive_property))
        .route("/api/units", get(list_units).post(create_unit))
        .route("/api/units/{id}", get(get_unit).patch(patch_unit))
        .route("/api/units/{id}/archive", post(archive_unit))
        .route("/api/tenants", get(list_tenants).post(create_tenant))
        .route("/api/tenants/{id}", get(get_tenant).patch(patch_tenant))
        .route("/api/tenants/{id}/merge", post(merge_tenant))
        .route("/api/tenants/{id}/archive", post(archive_tenant))
        .route("/api/tenants/{id}/id-document", get(tenant_id_document))
        .route("/api/owners", get(list_owners).post(create_owner))
        .route("/api/owners/{id}", get(get_owner).patch(patch_owner))
        .route("/api/owners/{id}/archive", post(archive_owner))
        .route("/api/search", get(search))
}

fn service(state: &AppState, ctx: &Ctx) -> Result<IdentityService, ApiError> {
    state.identity.clone().ok_or_else(|| ctx.unexpected())
}

fn parse_id(ctx: &Ctx, raw: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw.trim()).map_err(|_| ctx.err(ApiError::not_found()))
}

fn if_match(headers: &HeaderMap) -> Option<i32> {
    headers
        .get("if-match")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().trim_start_matches("W/").trim_matches('"'))
        .and_then(|v| v.parse().ok())
}

/// Le corps est un objet dont toutes les clés sont permises ; sinon `400 request.unknown_field`.
fn body_of(
    ctx: &Ctx,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
    allowed: &[&str],
) -> Result<Input, ApiError> {
    let ApiJson(map) = body?;
    if map.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err(ctx.err(ApiError::unknown_field()));
    }
    Ok(map)
}

/// Un objet imbriqué (`single_unit`, `backup_contact`) n'accepte que ses clés.
fn nested_ok(ctx: &Ctx, map: &Input, field: &str, allowed: &[&str]) -> Result<(), ApiError> {
    if let Some(Value::Object(inner)) = map.get(field)
        && inner.keys().any(|k| !allowed.contains(&k.as_str()))
    {
        return Err(ctx.err(ApiError::unknown_field()));
    }
    Ok(())
}

/// Filtres d'une liste : un filtre inconnu répond `422 request.filter_unknown` (API_CONTRACTS.md §2.5).
fn list_query(ctx: &Ctx, raw: &HashMap<String, String>, allowed: &[&str]) -> Result<ListQuery, ApiError> {
    if raw
        .keys()
        .any(|k| !allowed.contains(&k.as_str()) && k != "cursor" && k != "limit")
    {
        return Err(ctx.err(ApiError::filter_unknown()));
    }
    let uuid = |k: &str| -> Result<Option<Uuid>, ApiError> {
        raw.get(k)
            .map(|v| Uuid::parse_str(v.trim()).map_err(|_| ctx.err(ApiError::validation_reason("filter_value"))))
            .transpose()
    };
    let archived = match raw.get("archived").map(String::as_str) {
        None | Some("false") => false,
        Some("true") => true,
        Some(_) => return Err(ctx.err(ApiError::validation_reason("filter_value"))),
    };
    let limit = raw
        .get("limit")
        .map(|v| {
            v.parse::<i64>()
                .map_err(|_| ctx.err(ApiError::validation_reason("limit")))
        })
        .transpose()?;
    Ok(ListQuery {
        q: raw.get("q").cloned(),
        archived,
        district: raw.get("district").cloned(),
        owner_id: uuid("owner_id")?,
        property_id: uuid("property_id")?,
        usage: raw.get("usage").cloned(),
        kind: raw.get("kind").cloned(),
        status: raw.get("status").cloned(),
        cursor: raw.get("cursor").cloned(),
        limit,
    })
}

fn etag(version: i32) -> HeaderMap {
    let mut h = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&format!("\"{version}\"")) {
        h.insert(ETAG, v);
    }
    h
}

// ───────── corps des réponses ─────────

#[derive(Debug, Serialize, ToSchema)]
pub struct OwnerRefBody {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnitBody {
    pub id: String,
    pub property_id: String,
    pub property_name: String,
    pub reference: String,
    pub kind: String,
    pub usage: String,
    /// Texte décimal (« 18.50 »).
    pub surface_m2: Option<String>,
    /// Entier FCFA.
    pub reference_rent: Option<i64>,
    pub permitted_activity: Option<String>,
    /// `vacant`, `under_works` ou `archived`.
    pub status: String,
    pub archived_at: Option<String>,
    pub version: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PropertyBody {
    pub id: String,
    pub name: String,
    pub address: Option<String>,
    pub district: Option<String>,
    pub city: Option<String>,
    pub default_owner: Option<OwnerRefBody>,
    pub notes: Option<String>,
    pub unit_count: i64,
    pub vacant_count: i64,
    pub archived_at: Option<String>,
    pub version: i32,
    /// Lots du bien (fiche seulement ; vide dans une liste).
    pub units: Vec<UnitBody>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ContactBody {
    pub name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TenantBody {
    pub id: String,
    pub kind: String,
    pub last_name: String,
    pub first_name: Option<String>,
    pub display_name: String,
    pub trade_activity: Option<String>,
    pub phone: Option<String>,
    pub phone_is_whatsapp: bool,
    pub email: Option<String>,
    /// Absent pour le caissier, l'agent terrain, le comptable et le collaborateur (API_CONTRACTS.md §4.5).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<ContactBody>)]
    pub backup_contact: Option<Option<ContactBody>>,
    /// Absent comme le contact de secours.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub notes: Option<Option<String>>,
    /// Lots du locataire (vide jusqu'aux baux, étape 06).
    pub unit_labels: Vec<String>,
    /// Solde (null jusqu'aux encaissements, étape 07).
    pub balance: Option<i64>,
    pub merged_into_id: Option<String>,
    pub archived_at: Option<String>,
    pub version: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OwnerBody {
    pub id: String,
    pub kind: String,
    pub display_name: String,
    pub is_self: bool,
    /// Réservé au gérant (et au propriétaire direct) ; absent sinon (DP-92).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub phone: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub payout_method: Option<Option<String>>,
    pub property_count: i64,
    pub archived_at: Option<String>,
    pub version: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PropertiesPage {
    pub items: Vec<PropertyBody>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnitsPage {
    pub items: Vec<UnitBody>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TenantsPage {
    pub items: Vec<TenantBody>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OwnersPage {
    pub items: Vec<OwnerBody>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchTenantBody {
    pub id: String,
    pub display_name: String,
    pub phone: Option<String>,
    pub unit_labels: Vec<String>,
    pub balance: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchUnitBody {
    pub id: String,
    pub label: String,
    pub property_name: String,
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchBody {
    pub tenants: Vec<SearchTenantBody>,
    pub units: Vec<SearchUnitBody>,
    /// Vide jusqu'aux quittances (étape 08).
    #[schema(value_type = Vec<Object>)]
    pub receipts: Vec<Value>,
}

// ───────── corps des requêtes (documentation du contrat ; les clés sont contrôlées par `body_of`) ─────────

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct SingleUnitPayload {
    /// Référence du lot ; `1` par défaut.
    pub reference: Option<String>,
    /// `house` par défaut.
    pub kind: Option<String>,
    /// `residential` par défaut.
    pub usage: Option<String>,
    pub surface_m2: Option<String>,
    pub reference_rent: Option<i64>,
    pub permitted_activity: Option<String>,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct PropertyPayload {
    pub name: Option<String>,
    pub address: Option<String>,
    pub district: Option<String>,
    pub city: Option<String>,
    pub default_owner_id: Option<String>,
    pub notes: Option<String>,
    /// Maison seule : crée aussi 1 lot (R-010) ; création seulement.
    pub single_unit: Option<SingleUnitPayload>,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct UnitPayload {
    /// Création seulement : un lot ne change pas de bien.
    pub property_id: Option<String>,
    pub reference: Option<String>,
    pub kind: Option<String>,
    pub usage: Option<String>,
    /// Texte décimal, virgule ou point (« 18,5 »).
    pub surface_m2: Option<String>,
    /// Entier FCFA de 1 à 999 999 999.
    pub reference_rent: Option<i64>,
    /// Seulement pour un lot `commercial` ou `mixed` (R-012).
    pub permitted_activity: Option<String>,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct ContactPayload {
    pub name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct TenantPayload {
    pub kind: Option<String>,
    pub last_name: Option<String>,
    pub first_name: Option<String>,
    pub trade_activity: Option<String>,
    /// `+226` et 8 chiffres ; formes usuelles acceptées.
    pub phone: Option<String>,
    pub phone_is_whatsapp: Option<bool>,
    pub email: Option<String>,
    pub backup_contact: Option<ContactPayload>,
    pub notes: Option<String>,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct MergePayload {
    /// Fiche conservée.
    pub into_id: String,
}

#[derive(Debug, ToSchema)]
#[allow(dead_code)]
pub struct OwnerPayload {
    pub kind: Option<String>,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub payout_method: Option<String>,
}

const PROPERTY_FIELDS: [&str; 7] = [
    "name",
    "address",
    "district",
    "city",
    "default_owner_id",
    "notes",
    "single_unit",
];
const UNIT_FIELDS: [&str; 6] = [
    "reference",
    "kind",
    "usage",
    "surface_m2",
    "reference_rent",
    "permitted_activity",
];
const TENANT_FIELDS: [&str; 9] = [
    "kind",
    "last_name",
    "first_name",
    "trade_activity",
    "phone",
    "phone_is_whatsapp",
    "email",
    "backup_contact",
    "notes",
];
const OWNER_FIELDS: [&str; 4] = ["kind", "display_name", "phone", "payout_method"];

// ───────── conversion ─────────

fn time(t: Option<chrono::DateTime<chrono::Utc>>) -> Option<String> {
    t.map(rfc3339)
}

/// Conversion d'un lot pour les autres modules de routes (étape 06 : « en travaux »).
pub fn unit_body_pub(u: UnitView) -> UnitBody {
    unit_body(u)
}

fn unit_body(u: UnitView) -> UnitBody {
    UnitBody {
        id: u.id.to_string(),
        property_id: u.property_id.to_string(),
        property_name: u.property_name,
        reference: u.reference,
        kind: u.kind,
        usage: u.usage,
        surface_m2: u.surface_m2,
        reference_rent: u.reference_rent,
        permitted_activity: u.permitted_activity,
        status: u.status,
        archived_at: time(u.archived_at),
        version: u.version,
    }
}

fn property_body(p: PropertyView) -> PropertyBody {
    PropertyBody {
        id: p.id.to_string(),
        name: p.name,
        address: p.address,
        district: p.district,
        city: p.city,
        default_owner: p.default_owner.map(|o| OwnerRefBody {
            id: o.id.to_string(),
            display_name: o.display_name,
        }),
        notes: p.notes,
        unit_count: p.unit_count,
        vacant_count: p.vacant_count,
        archived_at: time(p.archived_at),
        version: p.version,
        units: p.units.into_iter().map(unit_body).collect(),
    }
}

fn tenant_body(t: TenantView) -> TenantBody {
    let (backup_contact, notes) = match t.private {
        Some(p) => (
            Some(p.backup_contact.map(|c| ContactBody {
                name: c.name,
                phone: c.phone,
            })),
            Some(p.notes),
        ),
        None => (None, None),
    };
    TenantBody {
        id: t.id.to_string(),
        kind: t.kind,
        last_name: t.last_name,
        first_name: t.first_name,
        display_name: t.display_name,
        trade_activity: t.trade_activity,
        phone: t.phone,
        phone_is_whatsapp: t.phone_is_whatsapp,
        email: t.email,
        backup_contact,
        notes,
        unit_labels: vec![],
        balance: None,
        merged_into_id: t.merged_into_id.map(|i| i.to_string()),
        archived_at: time(t.archived_at),
        version: t.version,
    }
}

fn owner_body(o: OwnerView) -> OwnerBody {
    let (phone, payout_method) = match o.private {
        Some(p) => (Some(p.phone), Some(p.payout_method)),
        None => (None, None),
    };
    OwnerBody {
        id: o.id.to_string(),
        kind: o.kind,
        display_name: o.display_name,
        is_self: o.is_self,
        phone,
        payout_method,
        property_count: o.property_count,
        archived_at: time(o.archived_at),
        version: o.version,
    }
}

fn search_body(s: SearchResult) -> SearchBody {
    SearchBody {
        tenants: s
            .tenants
            .into_iter()
            .map(|t| SearchTenantBody {
                id: t.id.to_string(),
                display_name: t.display_name,
                phone: t.phone,
                unit_labels: vec![],
                balance: None,
            })
            .collect(),
        units: s
            .units
            .into_iter()
            .map(|u| SearchUnitBody {
                id: u.id.to_string(),
                label: u.label,
                property_name: u.property_name,
                status: u.status,
            })
            .collect(),
        receipts: vec![],
    }
}

fn page_of<T, B>(p: Page<T>, f: impl Fn(T) -> B) -> (Vec<B>, Option<String>) {
    (p.items.into_iter().map(f).collect(), p.next_cursor)
}

type Created<T> = (StatusCode, HeaderMap, Json<T>);
type One<T> = (HeaderMap, Json<T>);

// ───────── biens ─────────

/// Biens (filtres `q`, `district`, `owner_id`, `archived`) ; l'agent terrain ne voit que les biens de ses lots.
#[utoipa::path(get, path = "/api/properties", responses((status = 200, description = "Biens", body = PropertiesPage)))]
pub async fn list_properties(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(raw): Query<HashMap<String, String>>,
) -> Result<Json<PropertiesPage>, ApiError> {
    session.require(Action::CatalogRead)?;
    let q = list_query(&session.ctx, &raw, &["q", "district", "owner_id", "archived"])?;
    let p = service(&state, &session.ctx)?
        .list_properties(&session.auth, &q)
        .await
        .map_err(|e| session.ctx.service(e))?;
    let (items, next_cursor) = page_of(p, property_body);
    Ok(Json(PropertiesPage { items, next_cursor }))
}

/// Crée un bien ; `single_unit` crée aussi son lot (maison seule, R-010).
#[utoipa::path(post, path = "/api/properties", request_body = PropertyPayload,
    responses((status = 201, description = "Bien créé", body = PropertyBody), (status = 422, description = "validation.failed")))]
pub async fn create_property(
    State(state): State<Arc<AppState>>,
    session: Session,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Created<PropertyBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let input = body_of(&session.ctx, body, &PROPERTY_FIELDS)?;
    nested_ok(&session.ctx, &input, "single_unit", &UNIT_FIELDS)?;
    let p = service(&state, &session.ctx)?
        .create_property(&session.auth, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((StatusCode::CREATED, etag(p.version), Json(property_body(p))))
}

/// Fiche d'un bien et ses lots.
#[utoipa::path(get, path = "/api/properties/{id}", params(("id" = String, Path)),
    responses((status = 200, description = "Bien", body = PropertyBody), (status = 404, description = "not_found")))]
pub async fn get_property(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<One<PropertyBody>, ApiError> {
    session.require(Action::CatalogRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let p = service(&state, &session.ctx)?
        .property(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(p.version), Json(property_body(p))))
}

/// Modifie un bien (`If-Match`).
#[utoipa::path(patch, path = "/api/properties/{id}", params(("id" = String, Path)), request_body = PropertyPayload,
    responses((status = 200, description = "Bien", body = PropertyBody), (status = 412, description = "concurrency.version_mismatch"),
              (status = 428, description = "concurrency.version_required"), (status = 409, description = "property.archived")))]
pub async fn patch_property(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<One<PropertyBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let fields: Vec<&str> = PROPERTY_FIELDS
        .iter()
        .copied()
        .filter(|f| *f != "single_unit")
        .collect();
    let input = body_of(&session.ctx, body, &fields)?;
    let p = service(&state, &session.ctx)?
        .patch_property(&session.auth, id, &input, if_match(&headers), &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(p.version), Json(property_body(p))))
}

/// Archive un bien et ses lots ; rien n'est supprimé (R-005).
#[utoipa::path(post, path = "/api/properties/{id}/archive", params(("id" = String, Path)),
    responses((status = 200, description = "Bien archivé", body = PropertyBody)))]
pub async fn archive_property(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<One<PropertyBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let p = service(&state, &session.ctx)?
        .archive_property(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(p.version), Json(property_body(p))))
}

// ───────── lots ─────────

/// Lots (filtres `property_id`, `usage`, `kind`, `status`, `q`, `archived`) ; l'agent terrain ne voit que ses lots.
#[utoipa::path(get, path = "/api/units", responses((status = 200, description = "Lots", body = UnitsPage)))]
pub async fn list_units(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(raw): Query<HashMap<String, String>>,
) -> Result<Json<UnitsPage>, ApiError> {
    session.require(Action::CatalogRead)?;
    let q = list_query(
        &session.ctx,
        &raw,
        &["q", "property_id", "usage", "kind", "status", "archived"],
    )?;
    let p = service(&state, &session.ctx)?
        .list_units(&session.auth, &q)
        .await
        .map_err(|e| session.ctx.service(e))?;
    let (items, next_cursor) = page_of(p, unit_body);
    Ok(Json(UnitsPage { items, next_cursor }))
}

/// Crée un lot (référence unique dans le bien, R-011 ; champs selon l'usage, R-012).
#[utoipa::path(post, path = "/api/units", request_body = UnitPayload,
    responses((status = 201, description = "Lot créé", body = UnitBody), (status = 409, description = "unit.reference_duplicate")))]
pub async fn create_unit(
    State(state): State<Arc<AppState>>,
    session: Session,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Created<UnitBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let mut fields = UNIT_FIELDS.to_vec();
    fields.push("property_id");
    let input = body_of(&session.ctx, body, &fields)?;
    let u = service(&state, &session.ctx)?
        .create_unit(&session.auth, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((StatusCode::CREATED, etag(u.version), Json(unit_body(u))))
}

/// Fiche d'un lot.
#[utoipa::path(get, path = "/api/units/{id}", params(("id" = String, Path)),
    responses((status = 200, description = "Lot", body = UnitBody), (status = 404, description = "not_found")))]
pub async fn get_unit(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<One<UnitBody>, ApiError> {
    session.require(Action::CatalogRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let u = service(&state, &session.ctx)?
        .unit(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(u.version), Json(unit_body(u))))
}

/// Modifie un lot (`If-Match`) ; `property_id` n'est pas modifiable.
#[utoipa::path(patch, path = "/api/units/{id}", params(("id" = String, Path)), request_body = UnitPayload,
    responses((status = 200, description = "Lot", body = UnitBody), (status = 412, description = "concurrency.version_mismatch"),
              (status = 409, description = "unit.archived, unit.reference_duplicate")))]
pub async fn patch_unit(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<One<UnitBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let input = body_of(&session.ctx, body, &UNIT_FIELDS)?;
    let u = service(&state, &session.ctx)?
        .patch_unit(&session.auth, id, &input, if_match(&headers), &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(u.version), Json(unit_body(u))))
}

/// Archive un lot (R-005).
#[utoipa::path(post, path = "/api/units/{id}/archive", params(("id" = String, Path)),
    responses((status = 200, description = "Lot archivé", body = UnitBody)))]
pub async fn archive_unit(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<One<UnitBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let u = service(&state, &session.ctx)?
        .archive_unit(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(u.version), Json(unit_body(u))))
}

// ───────── locataires ─────────

/// Locataires (filtres `q`, `archived`) ; champs masqués selon le rôle.
#[utoipa::path(get, path = "/api/tenants", responses((status = 200, description = "Locataires", body = TenantsPage)))]
pub async fn list_tenants(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(raw): Query<HashMap<String, String>>,
) -> Result<Json<TenantsPage>, ApiError> {
    session.require(Action::CatalogRead)?;
    let q = list_query(&session.ctx, &raw, &["q", "archived"])?;
    let p = service(&state, &session.ctx)?
        .list_tenants(&session.auth, &q)
        .await
        .map_err(|e| session.ctx.service(e))?;
    let (items, next_cursor) = page_of(p, tenant_body);
    Ok(Json(TenantsPage { items, next_cursor }))
}

/// Crée un locataire ; un numéro déjà porté répond `409 tenant.phone_duplicate` avec la fiche existante (R-017).
#[utoipa::path(post, path = "/api/tenants", request_body = TenantPayload,
    responses((status = 201, description = "Locataire créé", body = TenantBody), (status = 409, description = "tenant.phone_duplicate"),
              (status = 422, description = "validation.failed")))]
pub async fn create_tenant(
    State(state): State<Arc<AppState>>,
    session: Session,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Created<TenantBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let input = body_of(&session.ctx, body, &TENANT_FIELDS)?;
    nested_ok(&session.ctx, &input, "backup_contact", &["name", "phone"])?;
    let t = service(&state, &session.ctx)?
        .create_tenant(&session.auth, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((StatusCode::CREATED, etag(t.version), Json(tenant_body(t))))
}

/// Fiche d'un locataire (contact de secours et notes absents pour les rôles en lecture, §4.5).
#[utoipa::path(get, path = "/api/tenants/{id}", params(("id" = String, Path)),
    responses((status = 200, description = "Locataire", body = TenantBody), (status = 404, description = "not_found")))]
pub async fn get_tenant(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<One<TenantBody>, ApiError> {
    session.require(Action::CatalogRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let t = service(&state, &session.ctx)?
        .tenant(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(t.version), Json(tenant_body(t))))
}

/// Modifie un locataire (`If-Match`).
#[utoipa::path(patch, path = "/api/tenants/{id}", params(("id" = String, Path)), request_body = TenantPayload,
    responses((status = 200, description = "Locataire", body = TenantBody), (status = 409, description = "tenant.phone_duplicate, tenant.archived"),
              (status = 412, description = "concurrency.version_mismatch")))]
pub async fn patch_tenant(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<One<TenantBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let input = body_of(&session.ctx, body, &TENANT_FIELDS)?;
    nested_ok(&session.ctx, &input, "backup_contact", &["name", "phone"])?;
    let t = service(&state, &session.ctx)?
        .patch_tenant(&session.auth, id, &input, if_match(&headers), &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(t.version), Json(tenant_body(t))))
}

/// Fusionne cette fiche dans `into_id` : elle est archivée, son historique reste attaché (R-017, DP-92).
#[utoipa::path(post, path = "/api/tenants/{id}/merge", params(("id" = String, Path)), request_body = MergePayload,
    responses((status = 200, description = "Fiche conservée", body = TenantBody), (status = 409, description = "tenant.merge_invalid")))]
pub async fn merge_tenant(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<One<TenantBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let input = body_of(&session.ctx, body, &["into_id"])?;
    let t = service(&state, &session.ctx)?
        .merge_tenant(&session.auth, id, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(t.version), Json(tenant_body(t))))
}

/// Archive un locataire ; son numéro est libéré (R-005, R-017).
#[utoipa::path(post, path = "/api/tenants/{id}/archive", params(("id" = String, Path)),
    responses((status = 200, description = "Locataire archivé", body = TenantBody)))]
pub async fn archive_tenant(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<One<TenantBody>, ApiError> {
    session.require(Action::CatalogWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let t = service(&state, &session.ctx)?
        .archive_tenant(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(t.version), Json(tenant_body(t))))
}

/// Pièce d'identité : désactivée au pilote (`403 feature.disabled`, R-019).
#[utoipa::path(get, path = "/api/tenants/{id}/id-document", params(("id" = String, Path)),
    responses((status = 403, description = "feature.disabled")))]
pub async fn tenant_id_document(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    session.require(Action::CatalogWrite)?;
    service(&state, &session.ctx)?
        .tenant_id_document()
        .map_err(|e| session.ctx.service(e))?;
    Err(session.ctx.unexpected())
}

// ───────── mandants ─────────

/// Mandants (nom et type pour tous ; téléphone et mode de reversement pour le gérant, DP-92).
#[utoipa::path(get, path = "/api/owners", responses((status = 200, description = "Mandants", body = OwnersPage)))]
pub async fn list_owners(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(raw): Query<HashMap<String, String>>,
) -> Result<Json<OwnersPage>, ApiError> {
    session.require(Action::CatalogRead)?;
    let q = list_query(&session.ctx, &raw, &["q", "archived"])?;
    let p = service(&state, &session.ctx)?
        .list_owners(&session.auth, &q)
        .await
        .map_err(|e| session.ctx.service(e))?;
    let (items, next_cursor) = page_of(p, owner_body);
    Ok(Json(OwnersPage { items, next_cursor }))
}

/// Crée un mandant (gérant du plan Agence).
#[utoipa::path(post, path = "/api/owners", request_body = OwnerPayload,
    responses((status = 201, description = "Mandant créé", body = OwnerBody), (status = 403, description = "plan.feature_unavailable")))]
pub async fn create_owner(
    State(state): State<Arc<AppState>>,
    session: Session,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<Created<OwnerBody>, ApiError> {
    session.require(Action::MandateWrite)?;
    let input = body_of(&session.ctx, body, &OWNER_FIELDS)?;
    let o = service(&state, &session.ctx)?
        .create_owner(&session.auth, &input, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((StatusCode::CREATED, etag(o.version), Json(owner_body(o))))
}

/// Fiche d'un mandant.
#[utoipa::path(get, path = "/api/owners/{id}", params(("id" = String, Path)),
    responses((status = 200, description = "Mandant", body = OwnerBody), (status = 404, description = "not_found")))]
pub async fn get_owner(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<One<OwnerBody>, ApiError> {
    session.require(Action::CatalogRead)?;
    let id = parse_id(&session.ctx, &id)?;
    let o = service(&state, &session.ctx)?
        .owner(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(o.version), Json(owner_body(o))))
}

/// Modifie un mandant (`If-Match`).
#[utoipa::path(patch, path = "/api/owners/{id}", params(("id" = String, Path)), request_body = OwnerPayload,
    responses((status = 200, description = "Mandant", body = OwnerBody), (status = 409, description = "owner.archived"),
              (status = 412, description = "concurrency.version_mismatch")))]
pub async fn patch_owner(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Result<ApiJson<Map<String, Value>>, ApiError>,
) -> Result<One<OwnerBody>, ApiError> {
    session.require(Action::MandateWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let input = body_of(&session.ctx, body, &OWNER_FIELDS)?;
    let o = service(&state, &session.ctx)?
        .patch_owner(&session.auth, id, &input, if_match(&headers), &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(o.version), Json(owner_body(o))))
}

/// Archive un mandant (refusé s'il est propriétaire d'un bien non archivé, ou pour la fiche du propriétaire direct).
#[utoipa::path(post, path = "/api/owners/{id}/archive", params(("id" = String, Path)),
    responses((status = 200, description = "Mandant archivé", body = OwnerBody), (status = 409, description = "owner.has_properties, owner.is_self")))]
pub async fn archive_owner(
    State(state): State<Arc<AppState>>,
    session: Session,
    Path(id): Path<String>,
) -> Result<One<OwnerBody>, ApiError> {
    session.require(Action::MandateWrite)?;
    let id = parse_id(&session.ctx, &id)?;
    let o = service(&state, &session.ctx)?
        .archive_owner(&session.auth, id, &session.ctx.client)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok((etag(o.version), Json(owner_body(o))))
}

// ───────── recherche unique ─────────

/// Recherche unique (`q`, 2 à 80 caractères) : locataires, lots, quittances ; huit résultats au plus par groupe.
#[utoipa::path(get, path = "/api/search", params(("q" = String, Query)),
    responses((status = 200, description = "Résultats groupés", body = SearchBody), (status = 422, description = "validation.failed")))]
pub async fn search(
    State(state): State<Arc<AppState>>,
    session: Session,
    Query(raw): Query<HashMap<String, String>>,
) -> Result<Json<SearchBody>, ApiError> {
    session.require(Action::SearchUse)?;
    if raw.keys().any(|k| k != "q") {
        return Err(session.ctx.err(ApiError::filter_unknown()));
    }
    let q = raw.get("q").map(String::as_str).unwrap_or_default();
    let found = service(&state, &session.ctx)?
        .search(&session.auth, q)
        .await
        .map_err(|e| session.ctx.service(e))?;
    Ok(Json(search_body(found)))
}
