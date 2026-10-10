use hat_github_operator::{
    AdapterError, GitHubOperation, HatGitHubGrant, HatGitHubInvocation, HatGrantVerifier,
    RemoteExpectation, translate_authorized_at,
};
use hat_specifications::{
    ACTION_RESULT_SCHEMA, ActionReference, EvidenceReference, HatActionResult, HatInvocationOutcome,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use zixcel_github::{GitHubAction, GitHubApiRequest, GitHubRemoteExpectation};
use zixcel_github_egress_contracts::{
    GitHubAuthorizationV1, GitHubEgressCommandV1, GitHubEgressMode,
};

use crate::worker::Lease;
use crate::worker_io::{canonical_directory, message, path, required, run_hatter, write_document};
use crate::worker_transport;

const INPUT_SCHEMA: &str = "hathq://hat-github-operator/action-request/v1";
const OUTPUT_SCHEMA: &str = "hathq://hat-github-operator/action-receipt/v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerInput {
    schema: String,
    grant: HatGitHubGrant,
    invocation: HatGitHubInvocation,
    authorization: GitHubAuthorizationV1,
}

#[derive(Serialize)]
struct WorkerOutput {
    schema: &'static str,
    invocation_id: String,
    operation_id: String,
    authorization_digest_sha256: String,
    provider_request_id: String,
    remote_reference: String,
    remote_digest_sha256: Option<String>,
}

struct Verified;
impl HatGrantVerifier for Verified {
    fn verify(
        &self,
        _: &HatGitHubGrant,
        _: &HatGitHubInvocation,
        _: u64,
    ) -> Result<(), AdapterError> {
        Ok(())
    }
}

pub(super) fn process(
    args: &BTreeMap<String, String>,
    common: &[String],
    control: &Path,
    lease: &Lease,
) -> Result<(), String> {
    let outer = &lease.record.invocation;
    if outer.input.owner_id != "zixcel-graph" || outer.input.schema_id != INPUT_SCHEMA {
        return Err("invocation is outside the GitHub worker contract".into());
    }
    let input_store = canonical_directory(required(args, "input-store")?)?;
    let bytes = read_digest_document(&input_store, &outer.input)?;
    let value: WorkerInput = serde_json::from_slice(&bytes).map_err(message)?;
    validate_outer(&value, outer)?;
    let now = now_epoch_s()?;
    let expected_api = api_request(&value.invocation);
    let grant_json = serde_json::to_string(&value.grant).map_err(message)?;
    let invocation_json = serde_json::to_string(&value.invocation).map_err(message)?;
    let api_request_json = serde_json::to_string(&expected_api).map_err(message)?;
    let mut command = GitHubEgressCommandV1 {
        schema: "crowsi://provider-egress/github-command/v1".into(),
        mode: GitHubEgressMode::Verify,
        authorization: value.authorization.clone(),
        grant_json,
        invocation_json,
        api_request_json,
        now_epoch_s: now,
    };
    let verification = worker_transport::verify(args, &command)?;
    let translated = translate_authorized_at(&value.grant, &value.invocation, now, &Verified)
        .map_err(message)?;
    if serde_json::to_value(&translated).map_err(message)?
        != serde_json::to_value(&expected_api).map_err(message)?
    {
        return Err("GitHub adapter translation differs from the provider request".into());
    }
    command.mode = GitHubEgressMode::Execute;
    let receipt = worker_transport::execute(args, &command)?;
    let output = WorkerOutput {
        schema: OUTPUT_SCHEMA,
        invocation_id: outer.invocation_id.clone(),
        operation_id: outer.operation_id.clone(),
        authorization_digest_sha256: verification.authorization_digest_sha256.clone(),
        provider_request_id: receipt.provider_request_id,
        remote_reference: receipt.remote_reference,
        remote_digest_sha256: receipt.remote_digest_sha256,
    };
    let output_bytes = serde_json::to_vec(&output).map_err(message)?;
    let digest = hex::encode(Sha256::digest(&output_bytes));
    let output_store = canonical_directory(required(args, "output-store")?)?;
    write_document(&output_store.join(format!("{digest}.json")), &output_bytes)?;
    complete(
        args,
        common,
        control,
        lease,
        &digest,
        &verification.authorization_digest_sha256,
    )?;
    println!("{{\"processed\":true,\"outputDigestSha256\":\"{digest}\"}}");
    Ok(())
}

fn validate_outer(
    value: &WorkerInput,
    outer: &hat_specifications::HatInvocation,
) -> Result<(), String> {
    if value.schema != INPUT_SCHEMA
        || value.invocation.invocation_id != outer.invocation_id
        || value.invocation.request_id != outer.idempotency_key
        || value.invocation.package_id != outer.binding.package_id
        || value.invocation.package_digest_sha256 != outer.binding.package_digest_sha256
        || value.invocation.binding_digest_sha256 != outer.effective_grant.digest_sha256
        || value.grant.package_digest_sha256 != outer.binding.package_digest_sha256
        || value.grant.binding_digest_sha256 != outer.effective_grant.digest_sha256
        || operation_id(&value.invocation.operation) != outer.operation_id
    {
        return Err("GitHub worker input differs from the Hatter invocation".into());
    }
    Ok(())
}

fn api_request(value: &HatGitHubInvocation) -> GitHubApiRequest {
    GitHubApiRequest {
        schema: "zixcel://github/api-request/v1".into(),
        request_id: value.request_id.clone(),
        operation_id: value.invocation_id.clone(),
        connection_ref: value.connection_ref.clone(),
        owner: value.owner.clone(),
        repository: value.repository.clone(),
        action: action(&value.operation),
    }
}

fn action(value: &GitHubOperation) -> GitHubAction {
    match value {
        GitHubOperation::CreatePrivateRepository { default_branch } => {
            GitHubAction::CreatePrivateRepository {
                default_branch: default_branch.clone(),
            }
        }
        GitHubOperation::PushRepositorySnapshot {
            snapshot_ref,
            snapshot_digest_sha256,
            expected_remote,
        } => GitHubAction::PushRepositorySnapshot {
            snapshot_ref: snapshot_ref.clone(),
            snapshot_digest_sha256: snapshot_digest_sha256.clone(),
            expected_remote: match expected_remote {
                RemoteExpectation::Absent => GitHubRemoteExpectation::Absent,
                RemoteExpectation::Exact { commit_sha256 } => GitHubRemoteExpectation::Exact {
                    commit_sha256: commit_sha256.clone(),
                },
            },
        },
        GitHubOperation::DeleteRepository {
            expected_repository_id,
            backup_digest_sha256,
        } => GitHubAction::DeleteRepository {
            expected_repository_id: expected_repository_id.clone(),
            backup_digest_sha256: backup_digest_sha256.clone(),
        },
        GitHubOperation::CreateIssue {
            title,
            body_artifact_ref,
        } => GitHubAction::CreateIssue {
            title: title.clone(),
            body_artifact_ref: body_artifact_ref.clone(),
        },
        GitHubOperation::CreatePullRequest {
            base,
            head,
            title,
            body_artifact_ref,
        } => GitHubAction::CreatePullRequest {
            base: base.clone(),
            head: head.clone(),
            title: title.clone(),
            body_artifact_ref: body_artifact_ref.clone(),
        },
        GitHubOperation::DispatchWorkflow {
            workflow_ref,
            git_ref,
            inputs_artifact_ref,
        } => GitHubAction::DispatchWorkflow {
            workflow_ref: workflow_ref.clone(),
            git_ref: git_ref.clone(),
            inputs_artifact_ref: inputs_artifact_ref.clone(),
        },
    }
}

fn operation_id(value: &GitHubOperation) -> &'static str {
    match value {
        GitHubOperation::CreatePrivateRepository { .. } => {
            "hathq://vocabulary/action/create-private-github-repository/v1"
        }
        GitHubOperation::PushRepositorySnapshot { .. } => {
            "hathq://vocabulary/action/push-github-repository-snapshot/v1"
        }
        GitHubOperation::DeleteRepository { .. } => {
            "hathq://vocabulary/action/delete-github-repository/v1"
        }
        GitHubOperation::CreateIssue { .. } => "hathq://vocabulary/action/create-github-issue/v1",
        GitHubOperation::CreatePullRequest { .. } => {
            "hathq://vocabulary/action/create-github-pull-request/v1"
        }
        GitHubOperation::DispatchWorkflow { .. } => {
            "hathq://vocabulary/action/dispatch-github-workflow/v1"
        }
    }
}

fn complete(
    args: &BTreeMap<String, String>,
    common: &[String],
    control: &Path,
    lease: &Lease,
    digest: &str,
    authorization_digest: &str,
) -> Result<(), String> {
    let result = HatActionResult {
        schema: ACTION_RESULT_SCHEMA.into(),
        invocation_id: lease.record.invocation.invocation_id.clone(),
        operation_id: lease.record.invocation.operation_id.clone(),
        state_revision: lease.record.status.state_revision.saturating_add(1),
        projection_revision: lease
            .record
            .invocation
            .expected_projection_revision
            .saturating_add(1),
        outcome: HatInvocationOutcome::Completed,
        output: Some(ActionReference {
            owner_id: "hat-github-operator".into(),
            reference: digest.into(),
            schema_id: OUTPUT_SCHEMA.into(),
            digest_sha256: digest.into(),
        }),
        reason_id: None,
        evidence_refs: vec![EvidenceReference {
            owner_id: "crowsi".into(),
            reference: authorization_digest.into(),
            digest_sha256: authorization_digest
                .strip_prefix("sha256:")
                .unwrap_or(authorization_digest)
                .into(),
        }],
    };
    let result_path = control.join(format!("result-{}.json", result.invocation_id));
    write_document(&result_path, &serde_json::to_vec(&result).map_err(message)?)?;
    run_hatter(
        args,
        "complete",
        common,
        &[
            "--worker-id",
            required(args, "worker-id")?,
            "--result-json",
            path(&result_path)?,
        ],
    )?;
    Ok(())
}

fn read_digest_document(root: &Path, reference: &ActionReference) -> Result<Vec<u8>, String> {
    if reference.reference != reference.digest_sha256 {
        return Err("input is not content-addressed".into());
    }
    let bytes = fs::read(root.join(format!("{}.json", reference.reference))).map_err(message)?;
    if bytes.len() > 1_048_576 || hex::encode(Sha256::digest(&bytes)) != reference.digest_sha256 {
        return Err("input digest differs".into());
    }
    Ok(bytes)
}
fn now_epoch_s() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(message)
}
