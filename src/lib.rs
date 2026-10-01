#![forbid(unsafe_code)]
#![doc = "HAT-specific authorization adapter for the policy-neutral zixcel-github API."]

mod adapter;
mod boundary;
mod model;
mod operation;

pub use adapter::{HatGrantVerifier, translate_authorized_at};
pub use boundary::AdapterError;
pub use model::{HatGitHubGrant, HatGitHubInvocation, HatPermissionGrant, ZixcelGitHubApiRequest};
pub use operation::{GitHubOperation, RemoteExpectation};

pub const PACKAGE_JSON: &str = include_str!("../hat.package.json");
pub const PACKAGE_ID: &str = "hat/github-operator";
pub const REPOSITORY_ID: &str = "hat-github-operator";
pub const INVOCATION_SCHEMA: &str = "hathq://hat-github-operator/invocation/v1";
pub const GRANT_SCHEMA: &str = "hathq://hat-github-operator/runtime-grant/v1";
pub const ZIXCEL_API_REQUEST_SCHEMA: &str = "zixcel://github/api-request/v1";

pub const PERMISSIONS: [(&str, &str); 6] = [
    ("github-private-repository", "create-private-resource"),
    ("github-repository-content", "publish-artifact"),
    ("github-issue-creation", "create-work-item"),
    ("github-pull-request-creation", "create-review-request"),
    ("github-workflow", "dispatch-operation"),
    ("github-repository-administration", "delete-resource"),
];

#[must_use]
/// Returns the embedded immutable package document.
///
/// # Panics
///
/// Panics only when the package shipped in this crate is not valid JSON. The
/// package contract test prevents such a release.
pub fn package() -> serde_json::Value {
    serde_json::from_str(PACKAGE_JSON).expect("embedded GitHub HAT package must be valid JSON")
}
