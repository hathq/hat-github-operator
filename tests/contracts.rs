use std::collections::BTreeSet;

use hat_github_operator::{PACKAGE_ID, PERMISSIONS, REPOSITORY_ID, package};
use hat_specifications::{parse_package, validate_package};

#[test]
fn package_declares_only_the_closed_github_effect_set() {
    let package = package();
    assert_eq!(package["package_id"], PACKAGE_ID);
    assert_eq!(package["repository_id"], REPOSITORY_ID);
    let permissions = package["manifest"]["permissions"]
        .as_array()
        .expect("permissions")
        .iter()
        .map(|value| {
            (
                value["resource"].as_str().expect("resource"),
                value["operations"][0].as_str().expect("operation"),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(permissions, PERMISSIONS.into_iter().collect());
    assert!(
        package["manifest"]["permissions"]
            .as_array()
            .expect("permissions")
            .iter()
            .all(|value| value["mode"] == "execute")
    );
}

#[test]
fn package_has_no_public_force_or_credential_surface() {
    let wire = serde_json::to_string(&package())
        .expect("wire")
        .to_ascii_lowercase();
    for forbidden in ["public-repository", "force-push", "credential", "token"] {
        assert!(!wire.contains(forbidden), "{forbidden}");
    }
    assert!(
        package()["operations"]
            .as_array()
            .expect("operations")
            .iter()
            .all(|value| value["handler"]["kind"] == "hat-service"
                && value["handler"]["reference"] == "hat-github-operator")
    );
}

#[test]
fn package_declares_repository_information_and_exact_effects() {
    let package = package();
    let surfaces = package["information_surfaces"]
        .as_array()
        .expect("information surfaces");
    assert_eq!(surfaces.len(), 1);
    assert_eq!(surfaces[0]["canonical_type"], "domain.software.repository");
    assert_eq!(
        surfaces[0]["projection_schema"],
        "hathq://hat-github-operator/repository/v1"
    );
    assert_eq!(surfaces[0]["subject_relation"], "managed");

    let effects = package["operations"]
        .as_array()
        .expect("operations")
        .iter()
        .filter_map(|operation| operation.get("effects"))
        .flat_map(|value| value.as_array().expect("effects"))
        .map(|effect| {
            assert_eq!(effect["surface_id"], "github-repositories");
            effect["kind"].as_str().expect("effect kind")
        })
        .collect::<Vec<_>>();
    assert_eq!(effects, ["create", "update", "update"]);
}

#[test]
fn package_declares_github_connection_as_common_initial_setup() {
    let setup = &package()["setup_templates"][0];
    assert_eq!(setup["id"], "github-connection");
    assert_eq!(setup["context_namespace"], "github-connection");
    assert_eq!(
        setup["projection_schema"],
        "hathq://hat-github-operator/connection/v1"
    );
}

#[test]
fn package_speaks_the_exact_zixcel_github_protocol_for_every_operation() {
    let package = parse_package(hat_github_operator::PACKAGE_JSON).expect("valid package");
    assert!(
        validate_package(&package).valid,
        "{:?}",
        validate_package(&package)
    );
    let capability = package
        .communication_capabilities
        .first()
        .expect("communication capability");
    assert_eq!(capability.id, "zixcel-github-api");
    assert_eq!(capability.protocol.owner_id, "zixcel-github");
    assert_eq!(
        capability.protocol.protocol_id,
        hat_github_operator::ZIXCEL_API_REQUEST_SCHEMA
    );
    assert_eq!(capability.operation_ids.len(), package.operations.len());
}
