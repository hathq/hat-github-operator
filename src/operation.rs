use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RemoteExpectation {
    Absent,
    Exact { commit_sha256: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum GitHubOperation {
    CreatePrivateRepository {
        default_branch: String,
    },
    PushRepositorySnapshot {
        snapshot_ref: String,
        snapshot_digest_sha256: String,
        expected_remote: RemoteExpectation,
    },
    DeleteRepository {
        expected_repository_id: String,
        backup_digest_sha256: String,
    },
    CreateIssue {
        title: String,
        body_artifact_ref: String,
    },
    CreatePullRequest {
        base: String,
        head: String,
        title: String,
        body_artifact_ref: String,
    },
    DispatchWorkflow {
        workflow_ref: String,
        git_ref: String,
        inputs_artifact_ref: Option<String>,
    },
}

impl GitHubOperation {
    pub(crate) const fn required_permission(&self) -> (&'static str, &'static str) {
        match self {
            Self::CreatePrivateRepository { .. } => {
                ("github-private-repository", "create-private-resource")
            }
            Self::PushRepositorySnapshot { .. } => {
                ("github-repository-content", "publish-artifact")
            }
            Self::DeleteRepository { .. } => {
                ("github-repository-administration", "delete-resource")
            }
            Self::CreateIssue { .. } => ("github-issue-creation", "create-work-item"),
            Self::CreatePullRequest { .. } => {
                ("github-pull-request-creation", "create-review-request")
            }
            Self::DispatchWorkflow { .. } => ("github-workflow", "dispatch-operation"),
        }
    }
}
