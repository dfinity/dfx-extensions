//! Code for the command line: `dfx sns propose`
use clap::{ArgGroup, Parser};
use dfx_extensions_utils::call_extension_bundled_binary;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// SNS propose command line arguments.
#[derive(Debug, Parser)]
#[command(name("propose"))]
#[clap(
    group(
        ArgGroup::new("neuron-selection")
            .multiple(false)
            .required(true)
    )
)]
pub struct SnsProposeOpts {
    /// The id of the neuron with which to make the proposal. The current dfx
    /// identity must be able to operate this neuron (as a hotkey for instance).
    #[arg(long, group = "neuron-selection")]
    neuron_id: Option<u64>,

    /// The memo to use in conjunction with the current dfx identity to
    /// identify the neuron with which to make the proposal. This calculates
    /// the subaccount address with which the Neuron was created with. The current dfx
    /// identity must be able to operate this neuron (as a hotkey for instance).
    #[arg(long, group = "neuron-selection")]
    neuron_memo: Option<u64>,

    /// A test only alternative to `--neuron-id` and `--neuron-memo` that works
    /// on a local dfx server where an NNS is installed with the test neuron. If used
    /// with the mainnet version of the ic, `dfx sns propose` will return a failure.
    #[arg(long, default_value = "false", group = "neuron-selection")]
    test_neuron_proposer: bool,

    /// Path to a configuration file specifying the SNS to be created.
    init_config_file: PathBuf,
}

/// Executes the command line `dfx sns propose`.
pub fn exec(opts: SnsProposeOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let mut args = vec![OsString::from("propose")];

    let SnsProposeOpts {
        neuron_id,
        neuron_memo,
        test_neuron_proposer,
        init_config_file,
    } = opts;

    match (neuron_id, neuron_memo, test_neuron_proposer) {
        (Some(neuron_id), None, false) => args.extend(vec![
            OsString::from("--neuron-id"),
            OsString::from(neuron_id.to_string()),
        ]),
        (None, Some(neuron_memo), false) => args.extend(vec![
            OsString::from("--neuron-memo"),
            OsString::from(neuron_memo.to_string()),
        ]),
        (None, None, true) => args.extend(vec![OsString::from("--test-neuron-proposer")]),
        _ => {
            unreachable!("no neuron selected");
        }
    }

    args.push(OsString::from(
        init_config_file.to_string_lossy().to_string(),
    ));

    call_extension_bundled_binary("sns-cli", &args, dfx_cache_path)
}
