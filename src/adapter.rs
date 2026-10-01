use std::collections::BTreeSet;

use crate::boundary::{digest, github_name, identifier, reference};
use crate::{
    AdapterError, GRANT_SCHEMA, HatGitHubGrant, HatGitHubInvocation, INVOCATION_SCHEMA, PACKAGE_ID,
    PERMISSIONS, ZIXCEL_API_REQUEST_SCHEMA, ZixcelGitHubApiRequest,
};

pub trait HatGrantVerifier {
    /// Verifies the issuer, signature and immutable package binding.
    ///
    /// # Errors
    ///
    /// Returns an error unless the exact grant and invocation are trusted.
    fn verify(
        &self,
        grant: &HatGitHubGrant,
        invocation: &HatGitHubInvocation,
        now_epoch_s: u64,
    ) -> Result<(), AdapterError>;
}

/// Converts one exactly authorized HAT invocation into a policy-neutral API request.
///
/// # Errors
///
/// Returns an error for malformed, expired, out-of-scope or unverified input.
pub fn translate_authorized_at<V: HatGrantVerifier>(
    grant: &HatGitHubGrant,
    invocation: &HatGitHubInvocation,
    now_epoch_s: u64,
    verifier: &V,
) -> Result<ZixcelGitHubApiRequest, AdapterError> {
    validate_shape(grant, invocation)?;
    validate_exact_scope(grant, invocation, now_epoch_s)?;
    verifier.verify(grant, invocation, now_epoch_s)?;
    Ok(ZixcelGitHubApiRequest {
        schema: ZIXCEL_API_REQUEST_SCHEMA.into(),
        request_id: invocation.request_id.clone(),
        operation_id: invocation.invocation_id.clone(),
        connection_ref: invocation.connection_ref.clone(),
        owner: invocation.owner.clone(),
        repository: invocation.repository.clone(),
        action: invocation.operation.clone(),
    })
}

fn validate_shape(
    grant: &HatGitHubGrant,
    invocation: &HatGitHubInvocation,
) -> Result<(), AdapterError> {
    if grant.schema != GRANT_SCHEMA || invocation.schema != INVOCATION_SCHEMA {
        return Err(AdapterError::new("schema", "unexpected HAT GitHub schema"));
    }
    identifier("grant_id", &grant.grant_id)?;
    identifier("request_id", &invocation.request_id)?;
    identifier("invocation_id", &invocation.invocation_id)?;
    identifier("invocation.grant_id", &invocation.grant_id)?;
    reference("package_id", &grant.package_id)?;
    reference("invocation.package_id", &invocation.package_id)?;
    digest("package_digest_sha256", &grant.package_digest_sha256)?;
    digest("binding_digest_sha256", &grant.binding_digest_sha256)?;
    digest(
        "invocation.package_digest_sha256",
        &invocation.package_digest_sha256,
    )?;
    digest(
        "invocation.binding_digest_sha256",
        &invocation.binding_digest_sha256,
    )?;
    reference("connection_ref", &grant.connection_ref)?;
    reference("invocation.connection_ref", &invocation.connection_ref)?;
    github_name("owner", &grant.owner)?;
    github_name("invocation.owner", &invocation.owner)?;
    github_name("invocation.repository", &invocation.repository)?;
    validate_collections(grant)?;
    if grant.package_id != PACKAGE_ID
        || grant.revision == 0
        || grant.valid_until_epoch_s <= grant.valid_from_epoch_s
        || grant.valid_until_epoch_s - grant.valid_from_epoch_s > 300
    {
        return Err(AdapterError::new("grant", "invalid GitHub HAT grant"));
    }
    Ok(())
}

fn validate_exact_scope(
    grant: &HatGitHubGrant,
    invocation: &HatGitHubInvocation,
    now: u64,
) -> Result<(), AdapterError> {
    let permission = invocation.operation.required_permission();
    let permitted = grant
        .permissions
        .iter()
        .any(|value| value.resource == permission.0 && value.operation == permission.1);
    let repository = grant
        .repositories
        .iter()
        .any(|value| value.eq_ignore_ascii_case(&invocation.repository));
    if now < grant.valid_from_epoch_s
        || now >= grant.valid_until_epoch_s
        || invocation.grant_id != grant.grant_id
        || invocation.package_id != grant.package_id
        || invocation.package_digest_sha256 != grant.package_digest_sha256
        || invocation.binding_digest_sha256 != grant.binding_digest_sha256
        || invocation.connection_ref != grant.connection_ref
        || !invocation.owner.eq_ignore_ascii_case(&grant.owner)
        || !repository
        || !permitted
    {
        return Err(AdapterError::new(
            "grant",
            "invocation is outside the exact HAT grant",
        ));
    }
    Ok(())
}

fn validate_collections(grant: &HatGitHubGrant) -> Result<(), AdapterError> {
    if grant.repositories.is_empty()
        || grant.repositories.len() > 256
        || grant.permissions.is_empty()
        || grant.permissions.len() > PERMISSIONS.len()
    {
        return Err(AdapterError::new(
            "grant",
            "invalid bounded grant collection",
        ));
    }
    let mut repositories = BTreeSet::new();
    for value in &grant.repositories {
        github_name("repositories", value)?;
        if !repositories.insert(value.to_ascii_lowercase()) {
            return Err(AdapterError::new("repositories", "duplicate repository"));
        }
    }
    let mut permissions = BTreeSet::new();
    for value in &grant.permissions {
        let permission = (value.resource.as_str(), value.operation.as_str());
        if !PERMISSIONS.contains(&permission) || !permissions.insert(permission) {
            return Err(AdapterError::new(
                "permissions",
                "unsupported or duplicate permission",
            ));
        }
    }
    Ok(())
}
