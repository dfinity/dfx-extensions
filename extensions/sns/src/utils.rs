//! Utility functions for the SNS extension.

use anyhow::{anyhow, Result};
use ic_agent::agent::Agent;

/// Gets the agent corresponding to the given IC URL.
fn get_agent(ic_url: &str) -> Result<Agent> {
    Agent::builder()
        .with_url(ic_url)
        .build()
        .map_err(|e| anyhow!(e))
}

/// Gets the agent for IC mainnet.
pub fn get_mainnet_agent() -> Result<Agent> {
    let ic_url = "https://ic0.app/";
    get_agent(ic_url)
}
