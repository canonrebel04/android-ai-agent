//! Linux Tools Module for Android AI Agent
//!
//! This module provides a high-level interface for managing and executing
//! Linux tools within the sandbox environment. It integrates with LinuxSandbox
//! to provide tool-specific functionality and management.

use crate::linux_sandbox::{CommandResult, LinuxCommand, LinuxSandbox, SandboxConfig, SandboxError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Supported Linux tools
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinuxTool {
    /// Bash shell
    Bash,
    /// APT package manager
    Apt,
    /// cURL HTTP client
    Curl,
    /// Git version control
    Git,
    /// Python 3 interpreter
    Python3,
}

impl LinuxTool {
    /// Get the command name for this tool
    pub fn command_name(&self) -> &'static str {
        match self {
            LinuxTool::Bash => "bash",
            LinuxTool::Apt => "apt",
            LinuxTool::Curl => "curl",
            LinuxTool::Git => "git",
            LinuxTool::Python3 => "python3",
        }
    }

    /// Get a list of all supported tools
    pub fn all() -> Vec<LinuxTool> {
        vec![
            LinuxTool::Bash,
            LinuxTool::Apt,
            LinuxTool::Curl,
            LinuxTool::Git,
            LinuxTool::Python3,
        ]
    }

    /// Parse a string into a LinuxTool
    pub fn from_str(name: &str) -> Option<LinuxTool> {
        match name.to_lowercase().as_str() {
            "bash" | "sh" => Some(LinuxTool::Bash),
            "apt" | "apt-get" => Some(LinuxTool::Apt),
            "curl" => Some(LinuxTool::Curl),
            "git" => Some(LinuxTool::Git),
            "python3" | "python" => Some(LinuxTool::Python3),
            _ => None,
        }
    }
}

/// Tool execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteOptions {
    /// Timeout in seconds
    pub timeout: Option<u64>,
    /// Working directory
    pub workdir: Option<String>,
    /// Environment variables
    pub env_vars: Vec<(String, String)>,
    /// Whether to capture output
    pub capture_output: bool,
}

impl Default for ToolExecuteOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            workdir: None,
            env_vars: Vec::new(),
            capture_output: true,
        }
    }
}

/// Tool installation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInstallOptions {
    /// Version to install (if applicable)
    pub version: Option<String>,
    /// Whether to force reinstall
    pub force: bool,
    /// Additional flags for installation
    pub flags: Vec<String>,
}

impl Default for ToolInstallOptions {
    fn default() -> Self {
        Self {
            version: None,
            force: false,
            flags: Vec::new(),
        }
    }
}

/// Information about an installed tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// The tool name
    pub tool: LinuxTool,
    /// Whether the tool is installed
    pub installed: bool,
    /// Tool version (if available)
    pub version: Option<String>,
    /// Tool path (if available)
    pub path: Option<String>,
    /// Whether the tool is available in the sandbox
    pub available: bool,
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteResult {
    /// The tool that was executed
    pub tool: LinuxTool,
    /// The command that was executed
    pub command: String,
    /// Command result
    pub result: CommandResult,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Result of a tool installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInstallResult {
    /// The tool that was installed
    pub tool: LinuxTool,
    /// Whether installation succeeded
    pub success: bool,
    /// Installation message
    pub message: String,
    /// Installed version (if available)
    pub version: Option<String>,
    /// Command result (if applicable)
    pub command_result: Option<CommandResult>,
}

/// Error types for Linux tools operations
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum LinuxToolsError {
    #[error("Sandbox error: {0}")]
    SandboxError(#[from] SandboxError),
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Tool not installed: {0}")]
    ToolNotInstalled(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Installation failed: {0}")]
    InstallationFailed(String),
    #[error("Invalid tool: {0}")]
    InvalidTool(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Timeout error")]
    TimeoutError,
}

/// Manages Linux tools and their execution within the sandbox
pub struct LinuxToolManager {
    sandbox: LinuxSandbox,
    installed_tools: HashSet<LinuxTool>,
    tool_info: std::collections::HashMap<LinuxTool, ToolInfo>,
}

impl LinuxToolManager {
    /// Create a new LinuxToolManager with default configuration
    pub fn new() -> Self {
        Self {
            sandbox: LinuxSandbox::new(),
            installed_tools: HashSet::new(),
            tool_info: std::collections::HashMap::new(),
        }
    }

    /// Create a new LinuxToolManager with custom sandbox configuration
    pub fn with_config(config: SandboxConfig) -> Self {
        Self {
            sandbox: LinuxSandbox::with_config(config),
            installed_tools: HashSet::new(),
            tool_info: std::collections::HashMap::new(),
        }
    }

    /// Create a new LinuxToolManager with an existing sandbox
    pub fn with_sandbox(sandbox: LinuxSandbox) -> Self {
        Self {
            sandbox,
            installed_tools: HashSet::new(),
            tool_info: std::collections::HashMap::new(),
        }
    }

    /// Initialize the tool manager and check for installed tools
    pub fn init(&mut self) -> Result<(), LinuxToolsError> {
        self.sandbox.init()?;
        self.check_installed_tools()?;
        Ok(())
    }

    /// Get a reference to the underlying sandbox
    pub fn sandbox(&self) -> &LinuxSandbox {
        &self.sandbox
    }

    /// Get a mutable reference to the underlying sandbox
    pub fn sandbox_mut(&mut self) -> &mut LinuxSandbox {
        &mut self.sandbox
    }

    /// Check which tools are installed and available
    pub fn check_installed_tools(&mut self) -> Result<(), LinuxToolsError> {
        // For each tool, check if it's available in the sandbox
        for tool in LinuxTool::all() {
            let available = self.sandbox.is_command_allowed(tool.command_name());
            let installed = self.check_tool_installed(&tool)?;
            
            let info = ToolInfo {
                tool: tool.clone(),
                installed,
                version: self.get_tool_version(&tool).ok(),
                path: None,
                available,
            };
            
            self.tool_info.insert(tool, info);
            
            if installed && available {
                self.installed_tools.insert(tool);
            }
        }
        
        Ok(())
    }

    /// Check if a specific tool is installed
    pub fn check_tool_installed(&self, tool: &LinuxTool) -> Result<bool, LinuxToolsError> {
        // Try to execute a version check command
        let version_cmd = match tool {
            LinuxTool::Bash => "bash --version",
            LinuxTool::Apt => "apt --version",
            LinuxTool::Curl => "curl --version",
            LinuxTool::Git => "git --version",
            LinuxTool::Python3 => "python3 --version",
        };
        
        // Parse and prepare the command
        let command = LinuxSandbox::parse_command(version_cmd)
            .map_err(|e| LinuxToolsError::SandboxError(e))?;
        
        // Try to prepare the command (this checks if proot is available)
        match self.sandbox.prepare_command(&command) {
            Ok(_) => Ok(true), // Command can be prepared, assume installed
            Err(SandboxError::ProotNotAvailable) => Ok(false),
            Err(SandboxError::InvalidCommand(_)) => Ok(false),
            Err(_) => Ok(false), // Other errors, assume not installed
        }
    }

    /// Get the version of a tool
    pub fn get_tool_version(&self, tool: &LinuxTool) -> Result<String, LinuxToolsError> {
        let version_cmd = match tool {
            LinuxTool::Bash => "bash --version",
            LinuxTool::Apt => "apt --version",
            LinuxTool::Curl => "curl --version",
            LinuxTool::Git => "git --version",
            LinuxTool::Python3 => "python3 --version",
        };
        
        // For now, return a placeholder version
        // In a real implementation, this would execute the command and parse the output
        Ok(format!("{} version (placeholder)", tool.command_name()))
    }

    /// Execute a tool with the given arguments
    pub fn execute_tool(
        &self,
        tool: &LinuxTool,
        args: Vec<&str>,
        options: Option<ToolExecuteOptions>,
    ) -> Result<ToolExecuteResult, LinuxToolsError> {
        // Check if tool is available
        if !self.is_tool_available(tool)? {
            return Err(LinuxToolsError::ToolNotFound(format!(
                "Tool {} is not available",
                tool.command_name()
            )));
        }

        // Build the command based on the tool
        let command = match tool {
            LinuxTool::Bash => {
                if args.is_empty() {
                    LinuxCommand::Bash(String::new())
                } else {
                    LinuxCommand::Bash(args.join(" "))
                }
            }
            LinuxTool::Apt => LinuxCommand::Apt(args.iter().map(|s| s.to_string()).collect()),
            LinuxTool::Curl => {
                // Parse curl arguments
                let mut url = String::new();
                let mut output = None;
                let mut headers = Vec::new();
                let mut method = None;
                let mut data = None;
                let mut follow_redirects = false;
                let mut silent = false;
                
                let mut i = 0;
                while i < args.len() {
                    match args[i] {
                        "-o" | "--output" => {
                            if i + 1 < args.len() {
                                output = Some(args[i + 1].to_string());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "-H" | "--header" => {
                            if i + 1 < args.len() {
                                let header: Vec<&str> = args[i + 1].split(':').collect();
                                if header.len() == 2 {
                                    headers.push((header[0].to_string(), header[1].to_string()));
                                }
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "-X" | "--request" => {
                            if i + 1 < args.len() {
                                method = Some(args[i + 1].to_string());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "-d" | "--data" => {
                            if i + 1 < args.len() {
                                data = Some(args[i + 1].to_string());
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
                            if url.is_empty() && !args[i].starts_with('-') {
                                url = args[i].to_string();
                            }
                            i += 1;
                        }
                    }
                }
                
                if url.is_empty() && !args.is_empty() {
                    url = args[0].to_string();
                }
                
                LinuxCommand::Curl(crate::linux_sandbox::CurlCommand {
                    url,
                    output,
                    headers,
                    method,
                    data,
                    follow_redirects,
                    silent,
                })
            }
            LinuxTool::Git => {
                if args.is_empty() {
                    return Err(LinuxToolsError::ExecutionFailed(
                        "Git requires a subcommand".to_string(),
                    ));
                }
                let subcommand = args[0].to_string();
                let git_args = args[1..].iter().map(|s| s.to_string()).collect();
                LinuxCommand::Git(crate::linux_sandbox::GitCommand {
                    subcommand,
                    args: git_args,
                    repo_path: None,
                })
            }
            LinuxTool::Python3 => LinuxCommand::Python3(args.iter().map(|s| s.to_string()).collect()),
        };

        // Prepare the command for execution
        let cmd_str = self.sandbox.command_to_string(&command);
        
        // Prepare the command with proot
        let exec_cmd = self.sandbox.prepare_command(&command)?;
        
        // Create a result (in a real implementation, this would execute via Kotlin/JNI)
        // For now, we return a mock result indicating the command would be executed
        let result = CommandResult {
            stdout: format!("Command prepared: {}", exec_cmd),
            stderr: String::new(),
            exit_code: 0,
            success: true,
            command: cmd_str,
            duration_ms: 0,
        };
        
        Ok(ToolExecuteResult {
            tool: tool.clone(),
            command: cmd_str,
            result,
            metadata: serde_json::Value::Null,
        })
    }

    /// Execute a tool with a string command
    pub fn execute_tool_command(
        &self,
        tool: &LinuxTool,
        command: &str,
    ) -> Result<ToolExecuteResult, LinuxToolsError> {
        let args: Vec<&str> = command.split_whitespace().collect();
        self.execute_tool(tool, args, None)
    }

    /// Install a tool
    pub fn install_tool(
        &mut self,
        tool: &LinuxTool,
        options: Option<ToolInstallOptions>,
    ) -> Result<ToolInstallResult, LinuxToolsError> {
        let opts = options.unwrap_or_default();
        
        match tool {
            LinuxTool::Bash => {
                // Bash is typically pre-installed in most Linux environments
                Err(LinuxToolsError::InstallationFailed(
                    "Bash cannot be installed separately".to_string(),
                ))
            }
            LinuxTool::Apt => {
                // APT is typically pre-installed in Debian-based systems
                Err(LinuxToolsError::InstallationFailed(
                    "APT cannot be installed separately".to_string(),
                ))
            }
            LinuxTool::Curl => {
                // Install curl using apt
                let mut args = vec!["install", "-y", "curl"];
                if opts.force {
                    args.push("--reinstall");
                }
                args.extend(opts.flags);
                
                let command = LinuxCommand::Apt(args.iter().map(|s| s.to_string()).collect());
                let exec_cmd = self.sandbox.prepare_command(&command)?;
                
                // Mark as installed
                self.installed_tools.insert(LinuxTool::Curl);
                
                Ok(ToolInstallResult {
                    tool: LinuxTool::Curl,
                    success: true,
                    message: format!("curl installation prepared: {}", exec_cmd),
                    version: Some("latest".to_string()),
                    command_result: Some(CommandResult {
                        stdout: exec_cmd,
                        stderr: String::new(),
                        exit_code: 0,
                        success: true,
                        command: "apt install -y curl".to_string(),
                        duration_ms: 0,
                    }),
                })
            }
            LinuxTool::Git => {
                // Install git using apt
                let mut args = vec!["install", "-y", "git"];
                if opts.force {
                    args.push("--reinstall");
                }
                args.extend(opts.flags);
                
                let command = LinuxCommand::Apt(args.iter().map(|s| s.to_string()).collect());
                let exec_cmd = self.sandbox.prepare_command(&command)?;
                
                // Mark as installed
                self.installed_tools.insert(LinuxTool::Git);
                
                Ok(ToolInstallResult {
                    tool: LinuxTool::Git,
                    success: true,
                    message: format!("git installation prepared: {}", exec_cmd),
                    version: Some("latest".to_string()),
                    command_result: Some(CommandResult {
                        stdout: exec_cmd,
                        stderr: String::new(),
                        exit_code: 0,
                        success: true,
                        command: "apt install -y git".to_string(),
                        duration_ms: 0,
                    }),
                })
            }
            LinuxTool::Python3 => {
                // Install python3 using apt
                let mut args = vec!["install", "-y", "python3"];
                if opts.force {
                    args.push("--reinstall");
                }
                args.extend(opts.flags);
                
                let command = LinuxCommand::Apt(args.iter().map(|s| s.to_string()).collect());
                let exec_cmd = self.sandbox.prepare_command(&command)?;
                
                // Mark as installed
                self.installed_tools.insert(LinuxTool::Python3);
                
                Ok(ToolInstallResult {
                    tool: LinuxTool::Python3,
                    success: true,
                    message: format!("python3 installation prepared: {}", exec_cmd),
                    version: Some("latest".to_string()),
                    command_result: Some(CommandResult {
                        stdout: exec_cmd,
                        stderr: String::new(),
                        exit_code: 0,
                        success: true,
                        command: "apt install -y python3".to_string(),
                        duration_ms: 0,
                    }),
                })
            }
        }
    }

    /// Install a tool by name
    pub fn install_tool_by_name(
        &mut self,
        name: &str,
        options: Option<ToolInstallOptions>,
    ) -> Result<ToolInstallResult, LinuxToolsError> {
        let tool = LinuxTool::from_str(name)
            .ok_or_else(|| LinuxToolsError::InvalidTool(name.to_string()))?;
        self.install_tool(&tool, options)
    }

    /// List all installed tools
    pub fn list_installed_tools(&self) -> Vec<ToolInfo> {
        LinuxTool::all()
            .iter()
            .filter_map(|tool| self.tool_info.get(tool).cloned())
            .filter(|info| info.installed && info.available)
            .collect()
    }

    /// List all available tools (installed or not)
    pub fn list_available_tools(&self) -> Vec<ToolInfo> {
        LinuxTool::all()
            .iter()
            .filter_map(|tool| self.tool_info.get(tool).cloned())
            .filter(|info| info.available)
            .collect()
    }

    /// Check if a tool is available
    pub fn is_tool_available(&self, tool: &LinuxTool) -> bool {
        self.tool_info
            .get(tool)
            .map(|info| info.available)
            .unwrap_or(false)
    }

    /// Check if a tool is installed
    pub fn is_tool_installed(&self, tool: &LinuxTool) -> bool {
        self.installed_tools.contains(tool)
    }

    /// Get information about a specific tool
    pub fn get_tool_info(&self, tool: &LinuxTool) -> Option<&ToolInfo> {
        self.tool_info.get(tool)
    }

    /// Update package lists (for tools that can be installed via package manager)
    pub fn update_package_lists(&self) -> Result<CommandResult, LinuxToolsError> {
        let command = LinuxCommand::Apt(vec!["update".to_string()]);
        let cmd_str = self.sandbox.command_to_string(&command);
        let exec_cmd = self.sandbox.prepare_command(&command)?;
        
        Ok(CommandResult {
            stdout: format!("Package lists update prepared: {}", exec_cmd),
            stderr: String::new(),
            exit_code: 0,
            success: true,
            command: cmd_str,
            duration_ms: 0,
        })
    }

    /// Upgrade all installed packages
    pub fn upgrade_packages(&self) -> Result<CommandResult, LinuxToolsError> {
        let command = LinuxCommand::Apt(vec!["upgrade", "-y".to_string()]);
        let cmd_str = self.sandbox.command_to_string(&command);
        let exec_cmd = self.sandbox.prepare_command(&command)?;
        
        Ok(CommandResult {
            stdout: format!("Packages upgrade prepared: {}", exec_cmd),
            stderr: String::new(),
            exit_code: 0,
            success: true,
            command: cmd_str,
            duration_ms: 0,
        })
    }

    /// Execute bash command
    pub fn bash(&self, script: &str) -> Result<ToolExecuteResult, LinuxToolsError> {
        self.execute_tool_command(&LinuxTool::Bash, script)
    }

    /// Execute apt command
    pub fn apt(&self, args: Vec<&str>) -> Result<ToolExecuteResult, LinuxToolsError> {
        self.execute_tool(&LinuxTool::Apt, args, None)
    }

    /// Execute curl command
    pub fn curl(&self, url: &str) -> Result<ToolExecuteResult, LinuxToolsError> {
        self.execute_tool_command(&LinuxTool::Curl, url)
    }

    /// Execute git command
    pub fn git(&self, subcommand: &str, args: Vec<&str>) -> Result<ToolExecuteResult, LinuxToolsError> {
        let mut all_args = vec![subcommand];
        all_args.extend(args);
        self.execute_tool(&LinuxTool::Git, all_args, None)
    }

    /// Execute python3 command
    pub fn python3(&self, script: &str) -> Result<ToolExecuteResult, LinuxToolsError> {
        self.execute_tool_command(&LinuxTool::Python3, script)
    }
}

impl Default for LinuxToolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_tool_enum() {
        assert_eq!(LinuxTool::Bash.command_name(), "bash");
        assert_eq!(LinuxTool::Apt.command_name(), "apt");
        assert_eq!(LinuxTool::Curl.command_name(), "curl");
        assert_eq!(LinuxTool::Git.command_name(), "git");
        assert_eq!(LinuxTool::Python3.command_name(), "python3");
    }

    #[test]
    fn test_linux_tool_from_str() {
        assert_eq!(LinuxTool::from_str("bash"), Some(LinuxTool::Bash));
        assert_eq!(LinuxTool::from_str("BASH"), Some(LinuxTool::Bash));
        assert_eq!(LinuxTool::from_str("apt"), Some(LinuxTool::Apt));
        assert_eq!(LinuxTool::from_str("curl"), Some(LinuxTool::Curl));
        assert_eq!(LinuxTool::from_str("git"), Some(LinuxTool::Git));
        assert_eq!(LinuxTool::from_str("python3"), Some(LinuxTool::Python3));
        assert_eq!(LinuxTool::from_str("python"), Some(LinuxTool::Python3));
        assert_eq!(LinuxTool::from_str("unknown"), None);
    }

    #[test]
    fn test_linux_tool_all() {
        let all_tools = LinuxTool::all();
        assert_eq!(all_tools.len(), 5);
        assert!(all_tools.contains(&LinuxTool::Bash));
        assert!(all_tools.contains(&LinuxTool::Apt));
        assert!(all_tools.contains(&LinuxTool::Curl));
        assert!(all_tools.contains(&LinuxTool::Git));
        assert!(all_tools.contains(&LinuxTool::Python3));
    }

    #[test]
    fn test_tool_manager_creation() {
        let manager = LinuxToolManager::new();
        assert!(!manager.sandbox.is_proot_available());
    }

    #[test]
    fn test_tool_manager_with_config() {
        let config = SandboxConfig::default();
        let manager = LinuxToolManager::with_config(config);
        assert!(!manager.sandbox.is_proot_available());
    }

    #[test]
    fn test_execute_tool_bash() {
        let manager = LinuxToolManager::new();
        let result = manager.execute_tool(&LinuxTool::Bash, vec!["echo", "hello"], None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.tool, LinuxTool::Bash);
        assert!(result.result.success);
    }

    #[test]
    fn test_execute_tool_apt() {
        let manager = LinuxToolManager::new();
        let result = manager.execute_tool(&LinuxTool::Apt, vec!["install", "curl"], None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.tool, LinuxTool::Apt);
        assert!(result.result.success);
    }

    #[test]
    fn test_execute_tool_curl() {
        let manager = LinuxToolManager::new();
        let result = manager.execute_tool(&LinuxTool::Curl, vec!["https://example.com"], None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.tool, LinuxTool::Curl);
        assert!(result.result.success);
    }

    #[test]
    fn test_execute_tool_git() {
        let manager = LinuxToolManager::new();
        let result = manager.execute_tool(&LinuxTool::Git, vec!["clone", "https://github.com/test/repo.git"], None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.tool, LinuxTool::Git);
        assert!(result.result.success);
    }

    #[test]
    fn test_execute_tool_python3() {
        let manager = LinuxToolManager::new();
        let result = manager.execute_tool(&LinuxTool::Python3, vec!["-c", "print('hello')"], None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.tool, LinuxTool::Python3);
        assert!(result.result.success);
    }

    #[test]
    fn test_install_tool_curl() {
        let mut manager = LinuxToolManager::new();
        let result = manager.install_tool(&LinuxTool::Curl, None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.tool, LinuxTool::Curl);
        assert!(result.success);
    }

    #[test]
    fn test_install_tool_git() {
        let mut manager = LinuxToolManager::new();
        let result = manager.install_tool(&LinuxTool::Git, None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.tool, LinuxTool::Git);
        assert!(result.success);
    }

    #[test]
    fn test_install_tool_python3() {
        let mut manager = LinuxToolManager::new();
        let result = manager.install_tool(&LinuxTool::Python3, None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.tool, LinuxTool::Python3);
        assert!(result.success);
    }

    #[test]
    fn test_install_tool_by_name() {
        let mut manager = LinuxToolManager::new();
        let result = manager.install_tool_by_name("curl", None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.tool, LinuxTool::Curl);
    }

    #[test]
    fn test_list_installed_tools() {
        let mut manager = LinuxToolManager::new();
        // Initially, no tools should be marked as installed
        let installed = manager.list_installed_tools();
        assert!(installed.is_empty());
        
        // After checking, some tools might be available
        let _ = manager.check_installed_tools();
        let available = manager.list_available_tools();
        // The default config allows these tools
        assert!(!available.is_empty());
    }

    #[test]
    fn test_convenience_methods() {
        let manager = LinuxToolManager::new();
        
        let bash_result = manager.bash("echo hello");
        assert!(bash_result.is_ok());
        
        let apt_result = manager.apt(vec!["install", "curl"]);
        assert!(apt_result.is_ok());
        
        let curl_result = manager.curl("https://example.com");
        assert!(curl_result.is_ok());
        
        let git_result = manager.git("clone", vec!["https://github.com/test/repo.git"]);
        assert!(git_result.is_ok());
        
        let python_result = manager.python3("print('hello')");
        assert!(python_result.is_ok());
    }

    #[test]
    fn test_update_package_lists() {
        let manager = LinuxToolManager::new();
        let result = manager.update_package_lists();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_upgrade_packages() {
        let manager = LinuxToolManager::new();
        let result = manager.upgrade_packages();
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_tool_execute_options() {
        let options = ToolExecuteOptions::default();
        assert!(options.capture_output);
        assert!(options.timeout.is_none());
        assert!(options.workdir.is_none());
        assert!(options.env_vars.is_empty());
    }

    #[test]
    fn test_tool_install_options() {
        let options = ToolInstallOptions::default();
        assert!(!options.force);
        assert!(options.version.is_none());
        assert!(options.flags.is_empty());
    }

    #[test]
    fn test_tool_info() {
        let info = ToolInfo {
            tool: LinuxTool::Bash,
            installed: true,
            version: Some("1.0.0".to_string()),
            path: Some("/bin/bash".to_string()),
            available: true,
        };
        
        assert_eq!(info.tool, LinuxTool::Bash);
        assert!(info.installed);
        assert!(info.available);
    }

    #[test]
    fn test_error_types() {
        let err = LinuxToolsError::ToolNotFound("test".to_string());
        assert!(err.to_string().contains("test"));
        
        let err = LinuxToolsError::ToolNotInstalled("test".to_string());
        assert!(err.to_string().contains("test"));
        
        let err = LinuxToolsError::ExecutionFailed("test".to_string());
        assert!(err.to_string().contains("test"));
    }
}
