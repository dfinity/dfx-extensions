use anyhow::{bail, Context};
use dfx_core::extension::manifest::*;
use extension::*;
use std::{collections::BTreeMap, path::Path};

fn generate_extension_manifest(
    cmd: &clap::Command,
    old_manifest: ExtensionManifest,
) -> ExtensionManifest {
    ExtensionManifest {
        name: cmd.get_name().to_string(),
        summary: cmd.get_about().map(|a| a.to_string()).unwrap_or_default(),
        description: cmd.get_long_about().map(|a| a.to_string()),
        subcommands: Some(generate_subcommands(cmd)),
        ..old_manifest
    }
}

fn generate_subcommands(cmd: &clap::Command) -> ExtensionSubcommandsOpts {
    let mut subcommands = BTreeMap::new();

    for subcmd in cmd.get_subcommands() {
        subcommands.insert(
            subcmd.get_name().to_string(),
            ExtensionSubcommandOpts {
                about: subcmd.get_about().map(|a| a.to_string()),
                args: Some(generate_args(subcmd)),
                subcommands: if subcmd.has_subcommands() {
                    Some(generate_subcommands(subcmd))
                } else {
                    None
                },
            },
        );
    }

    ExtensionSubcommandsOpts(subcommands)
}

fn generate_args(cmd: &clap::Command) -> BTreeMap<String, ExtensionSubcommandArgOpts> {
    let mut args = BTreeMap::new();

    for arg in cmd.get_arguments() {
        args.insert(
            arg.get_id().to_string(),
            #[allow(deprecated)]
            ExtensionSubcommandArgOpts {
                about: arg.get_help().map(|h| h.to_string()),
                long: arg.get_long().map(|l| l.to_string()),
                short: arg.get_short(),
                multiple: false, // Deprecated, set to false
                values: match arg.get_num_args() {
                    None => ArgNumberOfValues::Number(if arg.get_action().takes_values() {
                        1
                    } else {
                        0
                    }),
                    Some(value_range) => {
                        let min = value_range.min_values();
                        let max = value_range.max_values();
                        if min == 0 && max == usize::MAX {
                            ArgNumberOfValues::Unlimited
                        } else if min == max {
                            ArgNumberOfValues::Number(min)
                        } else {
                            // max is inclusive, but ArgNumberOfValues::Range wants an exclusive range
                            ArgNumberOfValues::Range(min..(max.saturating_add(1)))
                        }
                    }
                },
            },
        );
    }

    args
}

pub fn verify_extension_manifest<Command: clap::CommandFactory>(path: &Path) -> anyhow::Result<()> {
    // read the manifest from the path and deserialize it
    let current_manifest_string = std::fs::read_to_string(path).context(format!(
        "Could not read the extension manifest at {}",
        path.display(),
    ))?;
    let current_manifest: ExtensionManifest = serde_json::from_str(&current_manifest_string)?;

    let command_info = Command::command();
    let updated_manifest = generate_extension_manifest(&command_info, current_manifest);

    let updated_manifest_string = serde_json::to_string_pretty(&updated_manifest)?;
    // write the json to the path
    if updated_manifest_string != current_manifest_string {
        std::fs::write(path, updated_manifest_string)?;
        bail!(
            "Extension manifest at {} was out of date. This has been fixed. Please commit the changes.", path.display()
        );
    }
    Ok(())
}
