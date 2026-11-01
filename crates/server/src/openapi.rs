//! Contrat d'API généré depuis le code (`openapi.json` versionné à la racine, vérifié par un test de fraîcheur).
//!
//! Outils (DP-84) : `utoipa` génère le contrat depuis le code Rust ; `oasdiff` détecte les ruptures en CI ; `openapi-typescript`
//! produit le client du front dans le même commit.

use utoipa::OpenApi;

use crate::api::{
    LoginBody, LoginPayload, MeBody, MembershipBody, OrganizationBody, OrganizationChoiceBody, OrganizationDetailBody,
    SelectOrganizationPayload, UserBody,
};
use crate::http::StatusBody;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Immo Suite API",
        description = "Contrat d'API d'Immo Suite (étape 02 : sondes, connexion, session, organisation active)",
        version = "0.0.0",
        license(name = "Propriétaire, tous droits réservés")
    ),
    paths(
        crate::http::health,
        crate::http::ready,
        crate::api::login,
        crate::api::logout,
        crate::api::logout_all,
        crate::api::me,
        crate::api::select_organization,
        crate::api::organization
    ),
    components(schemas(
        StatusBody,
        LoginPayload,
        LoginBody,
        UserBody,
        OrganizationChoiceBody,
        SelectOrganizationPayload,
        MeBody,
        MembershipBody,
        OrganizationBody,
        OrganizationDetailBody
    ))
)]
pub struct ApiDoc;

/// Le contrat au format JSON, indenté, avec un retour à la ligne final.
pub fn openapi_json() -> String {
    let mut s = ApiDoc::openapi().to_pretty_json().unwrap_or_default();
    s.push('\n');
    s
}
