//! La liste blanche des événements d'usage est versionnée (`tests/contract/events-whitelist.json`) : tout ajout, retrait ou changement
//! de propriété apparaît dans la revue de PR (F39-observation §4). Régénérer avec `UPDATE_WHITELIST=1 cargo test -p immo-app --test events_whitelist`.

use std::path::Path;

use immo_app::observation::events::whitelist_json;

#[test]
fn la_liste_blanche_versionnee_est_a_jour() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/contract/events-whitelist.json");
    let expected = whitelist_json();
    if std::env::var("UPDATE_WHITELIST").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &expected).unwrap();
        return;
    }
    let actual = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        actual, expected,
        "tests/contract/events-whitelist.json est périmé : UPDATE_WHITELIST=1 cargo test -p immo-app --test events_whitelist"
    );
}
