//! Utility functions for the SNS extension.
use anyhow::{Context, Result};
use dfx_core::interface::builder::IdentityPicker;
use dfx_core::DfxInterface;
use ic_agent::agent::Agent;

/// Gets an agent for a given network and identity. This is similar to the code DFX uses internally to get an agent.
/// If no identity is provided, it will use the identity currently selected in the DFX CLI.
pub async fn get_agent(network_name: &str, identity: Option<String>) -> Result<Agent> {
    let interface = dfx_interface(network_name, identity)
        .await
        .context("Failed to get dfx interface")?;
    Ok(interface.agent().clone())
}

/// Gets a dfx interface for a given network and identity. This is similar to the code DFX uses internally to build the interface.
/// So this function allows the DFX SNS Extension to use the same interface as DFX itself.
/// If no identity is provided, it will use the identity currently selected in the DFX CLI.
pub async fn dfx_interface(network_name: &str, identity: Option<String>) -> Result<DfxInterface> {
    let interface_builder = {
        let identity = identity
            .clone()
            .map(IdentityPicker::Named)
            .unwrap_or(IdentityPicker::Selected);
        DfxInterface::builder()
            .with_identity(identity)
            .with_network_named(network_name)
    };
    let interface = interface_builder.build().await.context(format!(
        "Failed to build dfx interface with network `{network_name}` and identity `{identity:?}`"
    ))?;
    if !interface.network_descriptor().is_ic {
        interface.agent().fetch_root_key().await.context(format!(
            "Failed to fetch root key from network `{network_name}`."
        ))?;
    }
    Ok(interface)
}
