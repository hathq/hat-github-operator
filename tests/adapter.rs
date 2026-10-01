use hat_github_operator::{
    AdapterError, GRANT_SCHEMA, GitHubOperation, HatGitHubGrant, HatGitHubInvocation,
    HatGrantVerifier, HatPermissionGrant, INVOCATION_SCHEMA, PACKAGE_ID, translate_authorized_at,
};

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

struct Verified;

impl HatGrantVerifier for Verified {
    fn verify(
        &self,
        _grant: &HatGitHubGrant,
        _invocation: &HatGitHubInvocation,
        _now_epoch_s: u64,
    ) -> Result<(), AdapterError> {
        Ok(())
    }
}

struct Rejected;

impl HatGrantVerifier for Rejected {
    fn verify(
        &self,
        _grant: &HatGitHubGrant,
        _invocation: &HatGitHubInvocation,
        _now_epoch_s: u64,
    ) -> Result<(), AdapterError> {
        Err(AdapterError {
            field: "signature",
            message: "unverified grant",
        })
    }
}

#[test]
fn exact_hat_grant_translates_to_policy_neutral_zixcel_request() {
    let provider = translate_authorized_at(&grant(), &invocation(), 150, &Verified)
        .expect("authorized translation");
    let value = serde_json::to_value(provider).expect("provider request");
    assert_eq!(value["schema"], "zixcel://github/api-request/v1");
    assert_eq!(value["operation_id"], "github-invocation-1");
    for field in [
        "grant_id",
        "package_id",
        "package_digest_sha256",
        "binding_digest_sha256",
    ] {
        assert!(value.get(field).is_none(), "{field}");
    }
}

#[test]
fn package_target_permission_time_and_signature_fail_closed() {
    for mutation in ["package", "binding", "repository", "permission", "expired"] {
        let grant = grant();
        let mut invocation = invocation();
        let now = match mutation {
            "package" => {
                invocation.package_id = "hat/other".into();
                150
            }
            "binding" => {
                invocation.binding_digest_sha256 = "b".repeat(64);
                150
            }
            "repository" => {
                invocation.repository = "other".into();
                150
            }
            "permission" => {
                invocation.operation = GitHubOperation::CreateIssue {
                    title: "Issue".into(),
                    body_artifact_ref: "artifact/issue".into(),
                };
                150
            }
            _ => 200,
        };
        assert!(translate_authorized_at(&grant, &invocation, now, &Verified).is_err());
    }
    assert!(translate_authorized_at(&grant(), &invocation(), 150, &Rejected).is_err());
}

fn grant() -> HatGitHubGrant {
    HatGitHubGrant {
        schema: GRANT_SCHEMA.into(),
        grant_id: "github-grant-1".into(),
        package_id: PACKAGE_ID.into(),
        package_digest_sha256: DIGEST.into(),
        binding_digest_sha256: DIGEST.into(),
        revision: 1,
        connection_ref: "github-connection-1".into(),
        owner: "example-org".into(),
        repositories: vec!["example-api".into()],
        permissions: vec![HatPermissionGrant {
            resource: "github-repository-content".into(),
            operation: "publish-artifact".into(),
        }],
        valid_from_epoch_s: 100,
        valid_until_epoch_s: 200,
    }
}

fn invocation() -> HatGitHubInvocation {
    HatGitHubInvocation {
        schema: INVOCATION_SCHEMA.into(),
        request_id: "github-request-1".into(),
        invocation_id: "github-invocation-1".into(),
        grant_id: "github-grant-1".into(),
        package_id: PACKAGE_ID.into(),
        package_digest_sha256: DIGEST.into(),
        binding_digest_sha256: DIGEST.into(),
        connection_ref: "github-connection-1".into(),
        owner: "example-org".into(),
        repository: "example-api".into(),
        operation: GitHubOperation::PushRepositorySnapshot {
            snapshot_ref: "artifact/source-snapshot-1".into(),
            snapshot_digest_sha256: DIGEST.into(),
            expected_remote: hat_github_operator::RemoteExpectation::Absent,
        },
    }
}

#[test]
fn deleting_a_repository_requires_an_independent_exact_grant() {
    let mut invocation = invocation();
    invocation.operation = GitHubOperation::DeleteRepository {
        expected_repository_id: "12345".into(),
        backup_digest_sha256: DIGEST.into(),
    };
    assert!(translate_authorized_at(&grant(), &invocation, 150, &Verified).is_err());
    let mut grant = grant();
    grant.permissions = vec![HatPermissionGrant {
        resource: "github-repository-administration".into(),
        operation: "delete-resource".into(),
    }];
    assert!(translate_authorized_at(&grant, &invocation, 150, &Verified).is_ok());
    assert!(translate_authorized_at(&grant, &invocation, 200, &Verified).is_err());
    invocation.repository = "other".into();
    assert!(translate_authorized_at(&grant, &invocation, 150, &Verified).is_err());
}
