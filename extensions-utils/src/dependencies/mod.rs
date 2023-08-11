use anyhow::anyhow;
use std::{
    env,
    ffi::OsStr,
    path::Path,
    process::{self, Command},
};

pub mod call;
pub mod dfx;
pub mod download_ic_binaries;
pub mod download_wasms;

pub fn execute_command(
    binary_path: &Path,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    dfx_cache_path: &Path,
) -> anyhow::Result<()> {
    let mut command = Command::new(binary_path);
    command.args(args);
    if let Some(old_path) = env::var_os("PATH") {
        let mut paths = env::split_paths(&old_path).collect::<Vec<_>>();
        paths.push(dfx_cache_path.to_path_buf());
        let new_path = env::join_paths(paths)?;
        command.env("PATH", new_path);
    } else {
        command.env("PATH", dfx_cache_path);
    }
    command.stdin(process::Stdio::null());
    command.stdout(process::Stdio::inherit());
    command.stderr(process::Stdio::inherit());

    let status = command
        .status()
        // e.g. "No such file or directory (os error 2)"
        .map_err(|e| {
            anyhow!(
                "Failed to execute binary at path '{}': {}",
                binary_path.display(),
                e
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        // running the command failed (exit code != 0)
        Err(anyhow!("Command execution failed: {:#?}", command))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::NamedTempFile;

    #[test]
    /// Create a temporary script that always succeeds
    fn test_execute_command_successful() {
        let mut temp_script = NamedTempFile::new().unwrap();
        writeln!(temp_script, "#!/bin/sh\nexit 0").unwrap();
        let path = temp_script.path().to_owned();
        if cfg!(unix) {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let args: Vec<String> = vec![];
        let result = execute_command(&path, args, &Path::new("."));
        assert!(result.is_ok());
    }

    #[test]
    /// Create a temporary script that always fails
    fn test_execute_command_fail() {
        let mut temp_script = NamedTempFile::new().unwrap();
        writeln!(temp_script, "#!/bin/sh\nexit 1").unwrap();
        let path = temp_script.path().to_owned();
        if cfg!(unix) {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let args: Vec<String> = vec!["arg".into()];
        let result = execute_command(&path, args, &Path::new("."));
        if let Err(e) = &result {
            assert_eq!(
                e.to_string(), //.split(':').next().unwrap(),
                format!(
                    r#"Command execution failed: Command {{
    program: "{binary}",
    args: [
        "{binary}",
        "arg",
    ],
    env: CommandEnv {{
        clear: false,
        vars: {{
            "PATH": Some(
                "{path}:.",
            ),
        }},
    }},
    stdin: Some(
        Null,
    ),
    stdout: Some(
        Inherit,
    ),
    stderr: Some(
        Inherit,
    ),
}}"#,
                    binary = path.display(),
                    path = env::var("PATH").unwrap()
                )
            );
        } else {
            panic!("Expected an error, but got {:?}", result);
        }
    }

    #[test]
    /// Try executing a non-existent command
    fn test_execute_command_nonexistent() {
        let args: Vec<String> = vec![];
        let result = execute_command(&Path::new("/nonexistent/binary"), args, &Path::new("."));
        if let Err(e) = &result {
            assert_eq!(
                e.to_string(),
                "Failed to execute binary at path '/nonexistent/binary': No such file or directory (os error 2)"
            );
        } else {
            panic!("Expected an error, but got {:?}", result);
        }
    }
}
