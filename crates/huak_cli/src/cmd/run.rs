use super::make_venv_command;
use huak_ops::{shell_name, Config, HuakResult};
use std::{env::consts::OS, process::Command};

pub fn run_command_str(command: &str, config: &Config) -> HuakResult<()> {
    let workspace = config.workspace();
    let python_env = workspace.current_python_environment()?;

    let mut cmd = Command::new(shell_name()?);
    let flag = match OS {
        "windows" => "/C",
        _ => "-c",
    };
    make_venv_command(&mut cmd, &python_env)?;
    cmd.args([flag, command]).current_dir(&config.cwd);
    config.terminal().run_command(&mut cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::test_fixtures::{test_config, test_resources_dir_path};
    use huak_ops::{copy_dir, env_path_string, CopyDirOptions, Verbosity};
    use tempfile::tempdir;

    #[test]
    fn test_run_command_str() {
        let dir = tempdir().unwrap();
        copy_dir(
            &test_resources_dir_path().join("mock-project"),
            &dir.path().join("mock-project"),
            &CopyDirOptions::default(),
        )
        .unwrap();
        let root = dir.path().join("mock-project");
        let cwd = root.to_path_buf();
        let config = test_config(&root, &cwd, Verbosity::Quiet);
        let ws = config.workspace();
        // For some reason this test fails with multiple threads used. Workspace.resolve_python_environment()
        // ends up updating the PATH environment variable causing subsequent Python searches using PATH to fail.
        // TODO
        let env_path = env_path_string().unwrap();
        let venv = ws.resolve_python_environment().unwrap();
        std::env::set_var("PATH", env_path);
        let venv_had_package = venv.contains_module("black").unwrap();

        run_command_str("pip install black", &config).unwrap();

        let venv_contains_package = venv.contains_module("black").unwrap();

        assert!(!venv_had_package);
        assert!(venv_contains_package);
    }
}