//! Linux Sandbox Module for Android AI Agent
//!
//! This module provides Rust-side functionality for executing commands
//! in a Linux sandbox environment using proot. It handles command parsing,
//! result processing, file operations, and package management.
//!
//! The Kotlin side handles the actual proot execution, while this Rust module
//! provides the command construction, parsing, and result handling.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Supported Linux commands
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinuxCommand {
    /// Bash shell command
    Bash(String),
    /// APT package manager command
    Apt(Vec<String>),
    /// cURL command
    Curl(CurlCommand),
    /// Git command
    Git(GitCommand),
    /// Python3 command
    Python3(Vec<String>),
    /// Custom command
    Custom(String, Vec<String>),
}

/// cURL command options
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CurlCommand {
    pub url: String,
    pub output: Option<String>,
    pub headers: Vec<(String, String)>,
    pub method: Option<String>,
    pub data: Option<String>,
    pub follow_redirects: bool,
    pub silent: bool,
}

/// Git command options
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GitCommand {
    pub subcommand: String,
    pub args: Vec<String>,
    pub repo_path: Option<String>,
}

/// Result of a command execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
    pub command: String,
    pub duration_ms: u64,
}

/// File operation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileResult {
    pub path: String,
    pub success: bool,
    pub message: String,
    pub size: Option<u64>,
    pub is_directory: bool,
}

/// Package operation result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageResult {
    pub package_name: String,
    pub operation: String,
    pub success: bool,
    pub message: String,
    pub version: Option<String>,
}

/// Error types for Linux sandbox operations
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum SandboxError {
    #[error("Command parsing error: {0}")]
    ParseError(String),
    #[error("Command execution error: {0}")]
    ExecutionError(String),
    #[error("File operation error: {0}")]
    FileError(String),
    #[error("Package operation error: {0}")]
    PackageError(String),
    #[error("Permission denied: {0}")]
    PermissionError(String),
    #[error("Timeout error")]
    TimeoutError,
    #[error("proot not available")]
    ProotNotAvailable,
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
}

/// Configuration for the Linux sandbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub rootfs_path: PathBuf,
    pub mount_point: PathBuf,
    pub timeout_seconds: u64,
    pub max_output_size: usize,
    pub allowed_commands: Vec<String>,
    pub blocked_commands: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            rootfs_path: PathBuf::from("/data/local/tmp/proot"),
            mount_point: PathBuf::from("/data/local/tmp/proot/mnt"),
            timeout_seconds: 30,
            max_output_size: 1024 * 1024, // 1MB
            allowed_commands: vec![
                "bash".to_string(),
                "apt".to_string(),
                "curl".to_string(),
                "git".to_string(),
                "python3".to_string(),
            ],
            blocked_commands: vec![
                "rm".to_string(),
                "dd".to_string(),
                "mkfs".to_string(),
                "fdisk".to_string(),
            ],
        }
    }
}

/// Main Linux Sandbox structure
pub struct LinuxSandbox {
    config: SandboxConfig,
    proot_available: bool,
}

impl LinuxSandbox {
    /// Create a new LinuxSandbox instance with default configuration
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
            proot_available: false,
        }
    }

    /// Create a new LinuxSandbox instance with custom configuration
    pub fn with_config(config: SandboxConfig) -> Self {
        Self {
            config,
            proot_available: false,
        }
    }

    /// Initialize the sandbox and check proot availability
    pub fn init(&mut self) -> Result<(), SandboxError> {
        // Check if proot is available (this would be called from Kotlin side)
        // For now, we assume it's available if we're on Android
        self.proot_available = true;
        Ok(())
    }

    /// Check if proot is available
    pub fn is_proot_available(&self) -> bool {
        self.proot_available
    }

    /// Parse a command string into a LinuxCommand
    pub fn parse_command(command: &str) -> Result<LinuxCommand, SandboxError> {
        let command = command.trim();
        
        if command.is_empty() {
            return Err(SandboxError::ParseError("Empty command".to_string()));
        }

        // Split into parts
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        match parts[0] {
            "bash" | "sh" | "/bin/bash" | "/bin/sh" => {
                let script = if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    String::new()
                };
                Ok(LinuxCommand::Bash(script))
            }
            "apt" | "apt-get" => {
                let args = parts[1..].iter().map(|s| s.to_string()).collect();
                Ok(LinuxCommand::Apt(args))
            }
            "curl" => {
                let mut url = String::new();
                let mut output = None;
                let mut headers = Vec::new();
                let mut method = None;
                let mut data = None;
                let mut follow_redirects = false;
                let mut silent = false;
                
                let mut i = 1;
                while i < parts.len() {
                    match parts[i] {
                        "-o" | "--output" => {
                            if i + 1 < parts.len() {
                                output = Some(parts[i + 1].to_string());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "-H" | "--header" => {
                            if i + 1 < parts.len() {
                                let header: Vec<&str> = parts[i + 1].split(':').collect();
                                if header.len() == 2 {
                                    headers.push((header[0].to_string(), header[1].to_string()));
                                }
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "-X" | "--request" => {
                            if i + 1 < parts.len() {
                                method = Some(parts[i + 1].to_string());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "-d" | "--data" => {
                            if i + 1 < parts.len() {
                                data = Some(parts[i + 1].to_string());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "-L" | "--location" => {
                            follow_redirects = true;
                            i += 1;
                        }
                        "-s" | "--silent" => {
                            silent = true;
                            i += 1;
                        }
                        _ => {
                            if url.is_empty() && !parts[i].starts_with('-') {
                                url = parts[i].to_string();
                            }
                            i += 1;
                        }
                    }
                }

                if url.is_empty() {
                    return Err(SandboxError::ParseError("No URL provided for curl".to_string()));
                }

                Ok(LinuxCommand::Curl(CurlCommand {
                    url,
                    output,
                    headers,
                    method,
                    data,
                    follow_redirects,
                    silent,
                }))
            }
            "git" => {
                if parts.len() < 2 {
                    return Err(SandboxError::ParseError("No git subcommand provided".to_string()));
                }
                
                let subcommand = parts[1].to_string();
                let args = parts[2..].iter().map(|s| s.to_string()).collect();
                
                Ok(LinuxCommand::Git(GitCommand {
                    subcommand,
                    args,
                    repo_path: None,
                }))
            }
            "python3" | "python" => {
                let args = parts[1..].iter().map(|s| s.to_string()).collect();
                Ok(LinuxCommand::Python3(args))
            }
            _ => {
                // Custom command
                let cmd = parts[0].to_string();
                let args = parts[1..].iter().map(|s| s.to_string()).collect();
                Ok(LinuxCommand::Custom(cmd, args))
            }
        }
    }

    /// Convert a LinuxCommand to a string for execution
    pub fn command_to_string(&self, command: &LinuxCommand) -> String {
        match command {
            LinuxCommand::Bash(script) => {
                if script.is_empty() {
                    "bash".to_string()
                } else {
                    format!("bash -c '{}'", script)
                }
            }
            LinuxCommand::Apt(args) => {
                if args.is_empty() {
                    "apt".to_string()
                } else {
                    format!("apt {}", args.join(" "))
                }
            }
            LinuxCommand::Curl(cmd) => {
                let mut parts = vec!["curl"];
                
                if cmd.follow_redirects {
                    parts.push("-L");
                }
                if cmd.silent {
                    parts.push("-s");
                }
                if let Some(ref output) = cmd.output {
                    parts.push("-o");
                    parts.push(output);
                }
                for (key, value) in &cmd.headers {
                    parts.push("-H");
                    parts.push(&format!("{}: {}", key, value));
                }
                if let Some(ref method) = cmd.method {
                    parts.push("-X");
                    parts.push(method);
                }
                if let Some(ref data) = cmd.data {
                    parts.push("-d");
                    parts.push(data);
                }
                parts.push(&cmd.url);
                
                parts.join(" ")
            }
            LinuxCommand::Git(cmd) => {
                let mut parts = vec!["git", &cmd.subcommand];
                parts.extend(&cmd.args);
                parts.join(" ")
            }
            LinuxCommand::Python3(args) => {
                if args.is_empty() {
                    "python3".to_string()
                } else {
                    format!("python3 {}", args.join(" "))
                }
            }
            LinuxCommand::Custom(cmd, args) => {
                if args.is_empty() {
                    cmd.clone()
                } else {
                    format!("{} {}", cmd, args.join(" "))
                }
            }
        }
    }

    /// Execute a command via proot (called from Kotlin side)
    /// This method constructs the command string and returns it for Kotlin to execute
    pub fn prepare_command(&self, command: &LinuxCommand) -> Result<String, SandboxError> {
        if !self.proot_available {
            return Err(SandboxError::ProotNotAvailable);
        }

        let cmd_str = self.command_to_string(command);
        
        // Add proot prefix for execution on Kotlin side
        // The Kotlin side will handle the actual proot execution
        Ok(format!("proot -S {} -b {}:/ {}", 
            self.config.rootfs_path.display(),
            self.config.mount_point.display(),
            cmd_str))
    }

    /// Process the result from a command execution
    pub fn process_result(&self, command: &str, stdout: &str, stderr: &str, exit_code: i32) -> CommandResult {
        let success = exit_code == 0;
        CommandResult {
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            exit_code,
            success,
            command: command.to_string(),
            duration_ms: 0, // Will be set by Kotlin side
        }
    }

    /// Execute a bash command
    pub fn execute_bash(&self, script: &str) -> Result<String, SandboxError> {
        let command = LinuxCommand::Bash(script.to_string());
        let cmd_str = self.prepare_command(&command)?;
        // In a real implementation, this would call through JNI to Kotlin
        // For now, return the command string that would be executed
        Ok(cmd_str)
    }

    /// Execute an apt command
    pub fn execute_apt(&self, args: Vec<&str>) -> Result<String, SandboxError> {
        let command = LinuxCommand::Apt(args.iter().map(|s| s.to_string()).collect());
        let cmd_str = self.prepare_command(&command)?;
        Ok(cmd_str)
    }

    /// Execute a curl command
    pub fn execute_curl(&self, url: &str) -> Result<String, SandboxError> {
        let command = LinuxCommand::Curl(CurlCommand {
            url: url.to_string(),
            output: None,
            headers: Vec::new(),
            method: None,
            data: None,
            follow_redirects: false,
            silent: false,
        });
        let cmd_str = self.prepare_command(&command)?;
        Ok(cmd_str)
    }

    /// Execute a git command
    pub fn execute_git(&self, subcommand: &str, args: Vec<&str>) -> Result<String, SandboxError> {
        let command = LinuxCommand::Git(GitCommand {
            subcommand: subcommand.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            repo_path: None,
        });
        let cmd_str = self.prepare_command(&command)?;
        Ok(cmd_str)
    }

    /// Execute a python3 command
    pub fn execute_python3(&self, script: &str) -> Result<String, SandboxError> {
        let command = LinuxCommand::Python3(vec!["-c".to_string(), script.to_string()]);
        let cmd_str = self.prepare_command(&command)?;
        Ok(cmd_str)
    }

    /// List files in a directory
    pub fn list_files(&self, path: &str) -> Result<String, SandboxError> {
        let script = format!("ls -la {}", path);
        self.execute_bash(&script)
    }

    /// Read a file
    pub fn read_file(&self, path: &str) -> Result<String, SandboxError> {
        let script = format!("cat {}", path);
        self.execute_bash(&script)
    }

    /// Write to a file
    pub fn write_file(&self, path: &str, content: &str) -> Result<String, SandboxError> {
        // Escape content for shell
        let escaped_content = content.replace("'", "'\\''");
        let script = format!("echo '{}' > {}", escaped_content, path);
        self.execute_bash(&script)
    }

    /// Create a directory
    pub fn create_directory(&self, path: &str) -> Result<String, SandboxError> {
        let script = format!("mkdir -p {}", path);
        self.execute_bash(&script)
    }

    /// Remove a file or directory
    pub fn remove(&self, path: &str, recursive: bool) -> Result<String, SandboxError> {
        let flag = if recursive { "-rf" } else { "-f" };
        let script = format!("rm {} {}", flag, path);
        self.execute_bash(&script)
    }

    /// Install a package using apt
    pub fn install_package(&self, package: &str) -> Result<String, SandboxError> {
        self.execute_apt(vec!["install", "-y", package])
    }

    /// Remove a package using apt
    pub fn remove_package(&self, package: &str) -> Result<String, SandboxError> {
        self.execute_apt(vec!["remove", "-y", package])
    }

    /// Update package lists
    pub fn update_packages(&self) -> Result<String, SandboxError> {
        self.execute_apt(vec!["update"])
    }

    /// Upgrade installed packages
    pub fn upgrade_packages(&self) -> Result<String, SandboxError> {
        self.execute_apt(vec!["upgrade", "-y"])
    }

    /// Check if a command is allowed
    pub fn is_command_allowed(&self, command: &str) -> bool {
        let base_cmd = command.split_whitespace().next().unwrap_or("");
        self.config.allowed_commands.contains(&base_cmd.to_string())
    }

    /// Check if a command is blocked
    pub fn is_command_blocked(&self, command: &str) -> bool {
        let base_cmd = command.split_whitespace().next().unwrap_or("");
        self.config.blocked_commands.contains(&base_cmd.to_string())
    }

    /// Get the sandbox configuration
    pub fn get_config(&self) -> &SandboxConfig {
        &self.config
    }

    /// Update the sandbox configuration
    pub fn update_config(&mut self, config: SandboxConfig) {
        self.config = config;
    }
}

/// JNI-compatible methods for Kotlin integration
#[cfg(target_os = "android")]
pub mod jni {
    use super::*;
    use jni::objects::{JClass, JString};
    use jni::sys::{jboolean, jint, jstring};
    use jni::EnvUnowned;

    /// Global sandbox instance for JNI calls
    static mut SANDBOX: Option<LinuxSandbox> = None;

    /// Initialize the sandbox
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_LinuxSandbox_nativeInit<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
        rootfs_path: JString<'local>,
        mount_point: JString<'local>,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            let rootfs: String = env.get_string(&rootfs_path)?.into();
            let mount: String = env.get_string(&mount_point)?.into();
            
            let mut sandbox = LinuxSandbox::new();
            sandbox.config.rootfs_path = PathBuf::from(rootfs);
            sandbox.config.mount_point = PathBuf::from(mount);
            
            match sandbox.init() {
                Ok(_) => {
                    unsafe { SANDBOX = Some(sandbox); }
                    let result = env.new_string("LinuxSandbox initialized")?;
                    Ok(result.into_raw())
                }
                Err(e) => {
                    let result = env.new_string(&format!("Error: {}", e))?;
                    Ok(result.into_raw())
                }
            }
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    /// Execute a command
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_LinuxSandbox_nativeExecute<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
        command: JString<'local>,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            let cmd: String = env.get_string(&command)?.into();
            
            let sandbox = unsafe { SANDBOX.as_ref() }
                .ok_or_else(|| jni::errors::Error::JavaException {
                    class: "java/lang/IllegalStateException".to_string(),
                    msg: "LinuxSandbox not initialized".to_string(),
                })?;

            match LinuxSandbox::parse_command(&cmd) {
                Ok(parsed_cmd) => {
                    match sandbox.prepare_command(&parsed_cmd) {
                        Ok(exec_cmd) => {
                            let result = env.new_string(&exec_cmd)?;
                            Ok(result.into_raw())
                        }
                        Err(e) => {
                            let result = env.new_string(&format!("Error: {}", e))?;
                            Ok(result.into_raw())
                        }
                    }
                }
                Err(e) => {
                    let result = env.new_string(&format!("Parse error: {}", e))?;
                    Ok(result.into_raw())
                }
            }
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    /// Check if proot is available
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_LinuxSandbox_nativeIsProotAvailable<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
    ) -> jboolean {
        let outcome = unowned_env.with_env(|_env| -> Result<_, jni::errors::Error> {
            let sandbox = unsafe { SANDBOX.as_ref() }
                .ok_or_else(|| jni::errors::Error::JavaException {
                    class: "java/lang/IllegalStateException".to_string(),
                    msg: "LinuxSandbox not initialized".to_string(),
                })?;
            
            Ok(if sandbox.is_proot_available() { 1 } else { 0 })
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    /// Process command result
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_LinuxSandbox_nativeProcessResult<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
        command: JString<'local>,
        stdout: JString<'local>,
        stderr: JString<'local>,
        exit_code: jint,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            let cmd: String = env.get_string(&command)?.into();
            let out: String = env.get_string(&stdout)?.into();
            let err: String = env.get_string(&stderr)?.into();
            
            let sandbox = unsafe { SANDBOX.as_ref() }
                .ok_or_else(|| jni::errors::Error::JavaException {
                    class: "java/lang/IllegalStateException".to_string(),
                    msg: "LinuxSandbox not initialized".to_string(),
                })?;

            let result = sandbox.process_result(&cmd, &out, &err, exit_code);
            let json = serde_json::to_string(&result).map_err(|e| {
                jni::errors::Error::JavaException {
                    class: "java/lang/IllegalStateException".to_string(),
                    msg: format!("Failed to serialize result: {}", e),
                }
            })?;
            
            let jresult = env.new_string(&json)?;
            Ok(jresult.into_raw())
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bash_command() {
        let cmd = "bash -c 'echo hello'";
        let parsed = LinuxSandbox::parse_command(cmd).unwrap();
        match parsed {
            LinuxCommand::Bash(script) => {
                assert_eq!(script, "-c 'echo hello'");
            }
            _ => panic!("Expected Bash command"),
        }
    }

    #[test]
    fn test_parse_apt_command() {
        let cmd = "apt install -y curl";
        let parsed = LinuxSandbox::parse_command(cmd).unwrap();
        match parsed {
            LinuxCommand::Apt(args) => {
                assert_eq!(args, vec!["install", "-y", "curl"]);
            }
            _ => panic!("Expected Apt command"),
        }
    }

    #[test]
    fn test_parse_curl_command() {
        let cmd = "curl -L -o output.txt https://example.com";
        let parsed = LinuxSandbox::parse_command(cmd).unwrap();
        match parsed {
            LinuxCommand::Curl(curl_cmd) => {
                assert_eq!(curl_cmd.url, "https://example.com");
                assert_eq!(curl_cmd.output, Some("output.txt".to_string()));
                assert!(curl_cmd.follow_redirects);
            }
            _ => panic!("Expected Curl command"),
        }
    }

    #[test]
    fn test_parse_git_command() {
        let cmd = "git clone https://github.com/test/repo.git";
        let parsed = LinuxSandbox::parse_command(cmd).unwrap();
        match parsed {
            LinuxCommand::Git(git_cmd) => {
                assert_eq!(git_cmd.subcommand, "clone");
                assert_eq!(git_cmd.args, vec!["https://github.com/test/repo.git"]);
            }
            _ => panic!("Expected Git command"),
        }
    }

    #[test]
    fn test_parse_python_command() {
        let cmd = "python3 -c 'print(hello)'";
        let parsed = LinuxSandbox::parse_command(cmd).unwrap();
        match parsed {
            LinuxCommand::Python3(args) => {
                assert_eq!(args, vec!["-c", "'print(hello)'"]);
            }
            _ => panic!("Expected Python3 command"),
        }
    }

    #[test]
    fn test_command_to_string() {
        let sandbox = LinuxSandbox::new();
        
        let cmd = LinuxCommand::Bash("echo hello".to_string());
        let result = sandbox.command_to_string(&cmd);
        assert_eq!(result, "bash -c 'echo hello'");

        let cmd = LinuxCommand::Apt(vec!["install", "curl"].iter().map(|s| s.to_string()).collect());
        let result = sandbox.command_to_string(&cmd);
        assert_eq!(result, "apt install curl");
    }

    #[test]
    fn test_process_result() {
        let sandbox = LinuxSandbox::new();
        let result = sandbox.process_result(
            "ls -la",
            "file1.txt\nfile2.txt",
            "",
            0
        );
        
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "file1.txt\nfile2.txt");
        assert_eq!(result.stderr, "");
    }

    #[test]
    fn test_is_command_allowed() {
        let sandbox = LinuxSandbox::new();
        
        assert!(sandbox.is_command_allowed("bash"));
        assert!(sandbox.is_command_allowed("apt"));
        assert!(sandbox.is_command_allowed("curl"));
        assert!(!sandbox.is_command_allowed("rm"));
    }

    #[test]
    fn test_is_command_blocked() {
        let sandbox = LinuxSandbox::new();
        
        assert!(sandbox.is_command_blocked("rm"));
        assert!(sandbox.is_command_blocked("dd"));
        assert!(!sandbox.is_command_blocked("bash"));
    }

    #[test]
    fn test_sandbox_config() {
        let config = SandboxConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_output_size, 1024 * 1024);
    }

    #[test]
    fn test_prepare_command() {
        let sandbox = LinuxSandbox::new();
        let cmd = LinuxCommand::Bash("echo hello".to_string());
        let result = sandbox.prepare_command(&cmd).unwrap();
        
        assert!(result.contains("proot"));
        assert!(result.contains("bash"));
    }

    #[test]
    fn test_execute_methods() {
        let sandbox = LinuxSandbox::new();
        
        let bash_result = sandbox.execute_bash("echo hello");
        assert!(bash_result.is_ok());
        
        let apt_result = sandbox.execute_apt(vec!["install", "curl"]);
        assert!(apt_result.is_ok());
        
        let curl_result = sandbox.execute_curl("https://example.com");
        assert!(curl_result.is_ok());
        
        let git_result = sandbox.execute_git("clone", vec!["https://github.com/test/repo.git"]);
        assert!(git_result.is_ok());
        
        let python_result = sandbox.execute_python3("print('hello')");
        assert!(python_result.is_ok());
    }

    #[test]
    fn test_file_operations() {
        let sandbox = LinuxSandbox::new();
        
        let list_result = sandbox.list_files("/tmp");
        assert!(list_result.is_ok());
        
        let read_result = sandbox.read_file("/tmp/test.txt");
        assert!(read_result.is_ok());
        
        let write_result = sandbox.write_file("/tmp/test.txt", "hello");
        assert!(write_result.is_ok());
        
        let mkdir_result = sandbox.create_directory("/tmp/test_dir");
        assert!(mkdir_result.is_ok());
        
        let rm_result = sandbox.remove("/tmp/test.txt", false);
        assert!(rm_result.is_ok());
    }

    #[test]
    fn test_package_operations() {
        let sandbox = LinuxSandbox::new();
        
        let install_result = sandbox.install_package("curl");
        assert!(install_result.is_ok());
        
        let remove_result = sandbox.remove_package("curl");
        assert!(remove_result.is_ok());
        
        let update_result = sandbox.update_packages();
        assert!(update_result.is_ok());
        
        let upgrade_result = sandbox.upgrade_packages();
        assert!(upgrade_result.is_ok());
    }
}
