use crowsi_provider_egress_contracts::{
    GitHubEgressCommandV1, GitHubEgressMode, GitHubEgressReceiptV1, GitHubVerificationReceiptV1,
    validate_github_command, validate_github_egress_receipt, validate_github_verification_receipt,
};
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

use crate::worker_io::{message, required, run_json_process};

pub(super) fn verify(
    args: &BTreeMap<String, String>,
    command: &GitHubEgressCommandV1,
) -> Result<GitHubVerificationReceiptV1, String> {
    if command.mode != GitHubEgressMode::Verify {
        return Err("verify mode required".into());
    }
    validate_github_command(command).map_err(str::to_owned)?;
    let value = exchange(args, command)?;
    validate_github_verification_receipt(&value, command).map_err(str::to_owned)?;
    Ok(value)
}

pub(super) fn execute(
    args: &BTreeMap<String, String>,
    command: &GitHubEgressCommandV1,
) -> Result<GitHubEgressReceiptV1, String> {
    if command.mode != GitHubEgressMode::Execute {
        return Err("execute mode required".into());
    }
    validate_github_command(command).map_err(str::to_owned)?;
    let value: GitHubEgressReceiptV1 = exchange(args, command)?;
    validate_github_egress_receipt(&value, command).map_err(str::to_owned)?;
    Ok(value)
}

fn exchange<T: DeserializeOwned>(
    args: &BTreeMap<String, String>,
    command: &GitHubEgressCommandV1,
) -> Result<T, String> {
    let input = serde_json::to_vec(command).map_err(message)?;
    let output = run_json_process(
        required(args, "crowsi-github-transport")?,
        required(args, "crowsi-transport-config")?,
        &input,
    )?;
    serde_json::from_slice(&output).map_err(message)
}
