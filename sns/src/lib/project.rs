use dfx_core::config::model::dfinity::Config;

pub struct ImportNetworkMapping {
    pub network_name_in_this_project: String,
    pub network_name_in_project_being_imported: String,
}

pub fn get_network_mappings(input: &[String]) -> anyhow::Result<Vec<ImportNetworkMapping>> {
    input
        .iter()
        .map(|v| {
            if let Some(index) = v.find('=') {
                if index == 0 {
                    Err(anyhow::anyhow!(
                        "malformed network mapping '{}': first network name is empty",
                        &v
                    ))
                } else if index == v.len() - 1 {
                    Err(anyhow::anyhow!(
                        "malformed network mapping '{}': second network name is empty",
                        &v
                    ))
                } else {
                    Ok(ImportNetworkMapping {
                        network_name_in_this_project: v[..index].to_string(),
                        network_name_in_project_being_imported: v[index + 1..].to_string(),
                    })
                }
            } else {
                Ok(ImportNetworkMapping {
                    network_name_in_this_project: v.clone(),
                    network_name_in_project_being_imported: v.clone(),
                })
            }
        })
        .collect()
}

pub async fn import_canister_definitions(
    logger: &slog::Logger,
    config: &mut Config,
    their_dfx_json_location: &str,
    prefix: Option<&str>,
    import_only_canister_name: Option<String>,
    network_mappings: &[ImportNetworkMapping],
) -> anyhow::Result<()> {
    Ok(())
}
