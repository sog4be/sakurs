//! Python environment detection and management
//!
//! This module provides utilities for detecting and managing Python environments
//! across different platforms and environment managers (venv, conda, uv, etc.).

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents different Python environment types
#[derive(Debug, Clone, PartialEq)]
pub enum PythonEnvironment {
    /// Standard virtual environment (venv)
    Venv(PathBuf),
    /// Conda environment
    Conda(String),
    /// UV Python environment
    Uv,
    /// System Python
    System(PathBuf),
}

/// Find the appropriate Python executable for the current environment
pub fn find_python_executable() -> Result<String, String> {
    let benchmarks_root = crate::paths::benchmarks_root()
        .map_err(|e| format!("Failed to get benchmarks root: {}", e))?;

    // Check environment variable first
    if let Ok(python_path) = env::var("SAKURS_PYTHON_PATH") {
        if Path::new(&python_path).exists() {
            return Ok(python_path);
        }
    }

    // Try different Python environments in order of preference
    let environments = vec![
        check_venv(&benchmarks_root),
        check_conda(),
        check_uv(),
        check_system_python(),
    ];

    if let Some(python_env) = environments.into_iter().flatten().next() {
        return Ok(get_python_command(&python_env));
    }

    Err("No suitable Python environment found. Please install Python 3.8+ or set SAKURS_PYTHON_PATH".to_string())
}

/// Check for a virtual environment in the project
fn check_venv(base_path: &Path) -> Result<PythonEnvironment, String> {
    let venv_paths = vec![
        base_path.join("venv/bin/python"),
        base_path.join("venv/Scripts/python.exe"),
        base_path.join(".venv/bin/python"),
        base_path.join(".venv/Scripts/python.exe"),
    ];

    for venv_path in venv_paths {
        if venv_path.exists() {
            return Ok(PythonEnvironment::Venv(venv_path));
        }
    }

    Err("No venv found".to_string())
}

/// Check for conda environment
fn check_conda() -> Result<PythonEnvironment, String> {
    // Check if we're in an active conda environment
    if let Ok(conda_env) = env::var("CONDA_DEFAULT_ENV") {
        if !conda_env.is_empty() && conda_env != "base" {
            // Verify conda is available
            if Command::new("conda").arg("--version").output().is_ok() {
                return Ok(PythonEnvironment::Conda(conda_env));
            }
        }
    }

    Err("No conda environment active".to_string())
}

/// Check for uv Python manager
fn check_uv() -> Result<PythonEnvironment, String> {
    if Command::new("uv").arg("--version").output().is_ok() {
        // Check if uv can find Python
        if Command::new("uv")
            .args(["run", "python", "--version"])
            .output()
            .is_ok()
        {
            return Ok(PythonEnvironment::Uv);
        }
    }

    Err("uv not available or no Python found".to_string())
}

/// Check for system Python
fn check_system_python() -> Result<PythonEnvironment, String> {
    let python_commands = if cfg!(windows) {
        vec!["python.exe", "python3.exe", "py.exe"]
    } else {
        vec!["python3", "python"]
    };

    for cmd in python_commands {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                return Ok(PythonEnvironment::System(PathBuf::from(cmd)));
            }
        }
    }

    Err("No system Python found".to_string())
}

/// Get the command to run Python for a given environment
fn get_python_command(env: &PythonEnvironment) -> String {
    match env {
        PythonEnvironment::Venv(path) => path.to_string_lossy().to_string(),
        PythonEnvironment::Conda(_) => "python".to_string(), // Conda activates the env
        PythonEnvironment::Uv => "uv run python".to_string(), // UV needs special handling
        PythonEnvironment::System(path) => path.to_string_lossy().to_string(),
    }
}

/// Build a Python command for the current environment
pub fn build_python_command() -> Result<Command, String> {
    let python_env = find_python_executable()?;

    let mut cmd = if python_env.contains("uv run") {
        let mut c = Command::new("uv");
        c.args(["run", "python"]);
        c
    } else {
        Command::new(python_env)
    };

    // Set Python environment variables for better compatibility
    cmd.env("PYTHONIOENCODING", "utf-8");

    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_environment_types() {
        let venv = PythonEnvironment::Venv(PathBuf::from("/path/to/venv/bin/python"));
        let conda = PythonEnvironment::Conda("myenv".to_string());
        let uv = PythonEnvironment::Uv;
        let system = PythonEnvironment::System(PathBuf::from("python3"));

        assert_eq!(get_python_command(&venv), "/path/to/venv/bin/python");
        assert_eq!(get_python_command(&conda), "python");
        assert_eq!(get_python_command(&uv), "uv run python");
        assert_eq!(get_python_command(&system), "python3");
    }
}
