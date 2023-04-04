use thiserror::Error;

#[derive(Error, Debug)]
pub enum DfxError {
    #[error("Failed to execute dfx as a command: {0}")]
    DfxExecutableError(std::io::Error),

    #[error("Failed to execute dfx as a command: {0}")]
    DfxVersionMalformed(semver::Error),

    #[error("Unexpected output from `dfx {command}`: {output}")]
    MalformedCommandOutput { command: String, output: String },
}
