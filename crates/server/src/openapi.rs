//! Contrat d'API généré depuis le code (`openapi.json` versionné à la racine, vérifié par un test de fraîcheur).
//!
//! Outils (DP-84) : `utoipa` génère le contrat depuis le code Rust ; `oasdiff` détecte les ruptures en CI ; `openapi-typescript`
//! produit le client du front dans le même commit.

use utoipa::OpenApi;

use crate::api::{
    LoginBody, LoginPayload, MeBody, MembershipBody, OrganizationBody, OrganizationChoiceBody, OrganizationDetailBody,
    SelectOrganizationPayload, UserBody,
};
use crate::api_account::{
    AcceptInvitationPayload, BackupCodesBody, ChangePasswordPayload, InvitationPreviewBody, ReauthPayload,
    ResetCompletePayload, ResetLinkBody, TotpConfirmPayload, TotpEnrollmentBody, TotpVerifyPayload, UpdateMePayload,
};
use crate::api_admin::{
    AuditActorBody, AuditActorChoice, AuditMetaBody, AuditObjectBody, AuditPageBody, AuditRowBody, ChangeRolePayload,
    CreateInvitationPayload, CreateSupportGrantPayload, CreatedInvitationBody, InvitationBody, InvitationsBody,
    MemberBody, MembersBody, PatchOrganizationPayload, PatchSettingsPayload, RoleSettingBody, RoleSettingPayload,
    RoleSettingsBody, SettingsBody, SupportAccessBody, SupportGrantBody, SupportOverviewBody, UnitAssignmentsBody,
    UnitAssignmentsPayload,
};
use crate::api_observation::{
    ClientErrorPayload, EventsPayload, FeedbackContextPayload, FeedbackCreatedBody, FeedbackPayload,
};
use crate::http::StatusBody;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Immo Suite API",
        description = "Contrat d'API d'Immo Suite (étapes 02 à 04 : sondes, connexion, session, comptes, membres, invitations, paramètres, accès support, journal, événements d'usage, erreurs du front, signalements)",
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
        crate::api::organization,
        crate::api_account::reauth,
        crate::api_account::totp_verify,
        crate::api_account::change_password,
        crate::api_account::update_me,
        crate::api_account::totp_begin,
        crate::api_account::totp_confirm,
        crate::api_account::issue_reset,
        crate::api_account::password_reset_complete,
        crate::api_account::invitation_preview,
        crate::api_account::accept_invitation,
        crate::api_admin::patch_organization,
        crate::api_admin::get_settings,
        crate::api_admin::patch_settings,
        crate::api_admin::list_users,
        crate::api_admin::change_role,
        crate::api_admin::disable_user,
        crate::api_admin::enable_user,
        crate::api_admin::set_unit_assignments,
        crate::api_admin::list_invitations,
        crate::api_admin::create_invitation,
        crate::api_admin::revoke_invitation,
        crate::api_admin::renew_invitation,
        crate::api_admin::get_role_settings,
        crate::api_admin::put_role_setting,
        crate::api_admin::list_support_grants,
        crate::api_admin::create_support_grant,
        crate::api_admin::revoke_support_grant,
        crate::api_admin::list_audit,
        crate::api_observation::post_events,
        crate::api_observation::post_client_error,
        crate::api_observation::post_feedback
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
        OrganizationDetailBody,
        ReauthPayload,
        TotpVerifyPayload,
        ChangePasswordPayload,
        UpdateMePayload,
        TotpConfirmPayload,
        TotpEnrollmentBody,
        BackupCodesBody,
        ResetCompletePayload,
        ResetLinkBody,
        InvitationPreviewBody,
        AcceptInvitationPayload,
        MemberBody,
        MembersBody,
        ChangeRolePayload,
        UnitAssignmentsPayload,
        UnitAssignmentsBody,
        InvitationBody,
        InvitationsBody,
        CreatedInvitationBody,
        CreateInvitationPayload,
        RoleSettingBody,
        RoleSettingsBody,
        RoleSettingPayload,
        PatchOrganizationPayload,
        SettingsBody,
        PatchSettingsPayload,
        SupportGrantBody,
        SupportAccessBody,
        SupportOverviewBody,
        CreateSupportGrantPayload,
        AuditActorBody,
        AuditObjectBody,
        AuditRowBody,
        AuditActorChoice,
        AuditMetaBody,
        AuditPageBody,
        EventsPayload,
        ClientErrorPayload,
        FeedbackContextPayload,
        FeedbackPayload,
        FeedbackCreatedBody
    ))
)]
pub struct ApiDoc;

/// Le contrat au format JSON, indenté, avec un retour à la ligne final.
pub fn openapi_json() -> String {
    let mut s = ApiDoc::openapi().to_pretty_json().unwrap_or_default();
    s.push('\n');
    s
}
