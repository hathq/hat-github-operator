use serde::{Deserialize, Serialize};

use crate::GitHubOperation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HatPermissionGrant {
    pub resource: String,
    pub operation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HatGitHubGrant {
    pub schema: String,
    pub grant_id: String,
    pub package_id: String,
    pub package_digest_sha256: String,
    pub binding_digest_sha256: String,
    pub revision: u64,
    pub connection_ref: String,
    pub owner: String,
    pub repositories: Vec<String>,
    pub permissions: Vec<HatPermissionGrant>,
    pub valid_from_epoch_s: u64,
    pub valid_until_epoch_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HatGitHubInvocation {
    pub schema: String,
    pub request_id: String,
    pub invocation_id: String,
    pub grant_id: String,
    pub package_id: String,
    pub package_digest_sha256: String,
    pub binding_digest_sha256: String,
    pub connection_ref: String,
    pub owner: String,
    pub repository: String,
    pub operation: GitHubOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZixcelGitHubApiRequest {
    pub schema: String,
    pub request_id: String,
    pub operation_id: String,
    pub connection_ref: String,
    pub owner: String,
    pub repository: String,
    pub action: GitHubOperation,
}
