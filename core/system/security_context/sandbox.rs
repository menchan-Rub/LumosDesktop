//! サンドボックスモジュール
//!
//! アプリケーションのサンドボックス環境を設定し、管理するためのモジュールです。
//! リソースの制限や分離されたコンテキストでのアプリケーション実行を実現します。

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};

/// サンドボックス設定
///
/// アプリケーションのサンドボックス環境の設定を定義します。
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// サンドボックスを有効にするかどうか
    pub enable_sandbox: bool,
    /// ネットワークアクセスを制限するかどうか
    pub restrict_network: bool,
    /// 許可されたネットワークホスト
    pub allowed_hosts: Vec<String>,
    /// 許可されたネットワークポート
    pub allowed_ports: Vec<u16>,
    /// ファイルアクセスを制限するかどうか
    pub restrict_file_access: bool,
    /// 許可されたファイルパス
    pub allowed_paths: Vec<PathBuf>,
    /// IPC（プロセス間通信）を制限するかどうか
    pub restrict_ipc: bool,
    /// 許可されたIPCターゲット
    pub allowed_ipc_targets: Vec<String>,
    /// デバイスアクセスを制限するかどうか
    pub restrict_devices: bool,
    /// 許可されたデバイス
    pub allowed_devices: Vec<String>,
    /// グラフィックス機能を制限するかどうか
    pub restrict_graphics: bool,
    /// メモリ使用量の制限（バイト単位、0は無制限）
    pub memory_limit: usize,
    /// CPU使用率の制限（0-100、0は無制限）
    pub cpu_limit: u8,
    /// システムコール制限を有効にするかどうか
    pub enable_syscall_filtering: bool,
    /// 許可されたシステムコール
    pub allowed_syscalls: Vec<String>,
    /// ケイパビリティ制限を有効にするかどうか
    pub enable_capabilities_filtering: bool,
    /// 許可されたケイパビリティ
    pub allowed_capabilities: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enable_sandbox: false,
            restrict_network: false,
            allowed_hosts: Vec::new(),
            allowed_ports: Vec::new(),
            restrict_file_access: false,
            allowed_paths: Vec::new(),
            restrict_ipc: false,
            allowed_ipc_targets: Vec::new(),
            restrict_devices: false,
            allowed_devices: Vec::new(),
            restrict_graphics: false,
            memory_limit: 0,
            cpu_limit: 0,
            enable_syscall_filtering: false,
            allowed_syscalls: Vec::new(),
            enable_capabilities_filtering: false,
            allowed_capabilities: Vec::new(),
        }
    }
}

impl SandboxConfig {
    /// 新しいサンドボックス設定を作成
    pub fn new(enable_sandbox: bool) -> Self {
        let mut config = Self::default();
        config.enable_sandbox = enable_sandbox;
        config
    }

    /// セキュリティプロファイルを適用
    pub fn apply_security_profile(&mut self, profile: SandboxSecurityProfile) {
        match profile {
            SandboxSecurityProfile::Minimal => {
                self.enable_sandbox = true;
                self.restrict_file_access = true;
                // 基本的なパスを許可
                self.allowed_paths = vec![
                    PathBuf::from("/tmp"),
                    PathBuf::from("/usr/share"),
                ];
            },
            SandboxSecurityProfile::Standard => {
                self.enable_sandbox = true;
                self.restrict_network = true;
                self.restrict_file_access = true;
                self.restrict_devices = true;
                
                // 基本的なホストを許可
                self.allowed_hosts = vec![
                    "localhost".to_string(),
                    "127.0.0.1".to_string(),
                ];
                
                // 一般的なポートを許可
                self.allowed_ports = vec![80, 443, 8000, 8080];
                
                // 基本的なパスを許可
                self.allowed_paths = vec![
                    PathBuf::from("/tmp"),
                    PathBuf::from("/usr/share"),
                ];
                
                // 基本的なデバイスを許可
                self.allowed_devices = vec![
                    "/dev/null".to_string(),
                    "/dev/zero".to_string(),
                    "/dev/urandom".to_string(),
                ];
            },
            SandboxSecurityProfile::Strict => {
                self.enable_sandbox = true;
                self.restrict_network = true;
                self.restrict_file_access = true;
                self.restrict_ipc = true;
                self.restrict_devices = true;
                self.restrict_graphics = true;
                self.enable_syscall_filtering = true;
                self.enable_capabilities_filtering = true;
                
                // 厳格なシステムコール制限
                self.allowed_syscalls = vec![
                    "read".to_string(),
                    "write".to_string(),
                    "open".to_string(),
                    "close".to_string(),
                    "stat".to_string(),
                    "fstat".to_string(),
                    "mmap".to_string(),
                    "munmap".to_string(),
                    "brk".to_string(),
                    "rt_sigaction".to_string(),
                    "rt_sigprocmask".to_string(),
                    "exit".to_string(),
                    "exit_group".to_string(),
                ];
                
                // 厳格なケイパビリティ制限
                self.allowed_capabilities = Vec::new();
            },
            SandboxSecurityProfile::Custom => {
                // カスタムプロファイルでは何もしない
            },
        }
    }

    /// ホストへのアクセスを許可するかどうかを確認
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if !self.restrict_network || !self.enable_sandbox {
            return true;
        }
        
        self.allowed_hosts.iter().any(|h| {
            if h.starts_with('*') {
                let suffix = &h[1..];
                host.ends_with(suffix)
            } else if h.ends_with('*') {
                let prefix = &h[..h.len() - 1];
                host.starts_with(prefix)
            } else {
                h == host
            }
        })
    }

    /// ポートへのアクセスを許可するかどうかを確認
    pub fn is_port_allowed(&self, port: u16) -> bool {
        if !self.restrict_network || !self.enable_sandbox {
            return true;
        }
        
        self.allowed_ports.contains(&port)
    }

    /// パスへのアクセスを許可するかどうかを確認
    pub fn is_path_allowed(&self, path: &Path) -> bool {
        if !self.restrict_file_access || !self.enable_sandbox {
            return true;
        }
        
        let path = if let Ok(canonical) = path.canonicalize() {
            canonical
        } else {
            return false; // パスが存在しない場合はアクセス拒否
        };
        
        self.allowed_paths.iter().any(|allowed| {
            if allowed.ends_with("..") {
                // ディレクトリとその中身すべてへのアクセスを許可
                let parent = allowed.parent().unwrap_or(allowed);
                path.starts_with(parent)
            } else {
                // 特定のパスへのアクセスのみを許可
                path == *allowed
            }
        })
    }

    /// デバイスへのアクセスを許可するかどうかを確認
    pub fn is_device_allowed(&self, device_path: &str) -> bool {
        if !self.restrict_devices || !self.enable_sandbox {
            return true;
        }
        
        self.allowed_devices.contains(&device_path.to_string())
    }

    /// IPCターゲットへのアクセスを許可するかどうかを確認
    pub fn is_ipc_target_allowed(&self, target: &str) -> bool {
        if !self.restrict_ipc || !self.enable_sandbox {
            return true;
        }
        
        self.allowed_ipc_targets.contains(&target.to_string())
    }
}

/// サンドボックスセキュリティプロファイル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxSecurityProfile {
    /// 最小限の制限（ファイルアクセスのみ制限）
    Minimal,
    /// 標準的な制限（ネットワーク、ファイル、デバイスを制限）
    Standard,
    /// 厳格な制限（すべての機能を厳しく制限）
    Strict,
    /// カスタムプロファイル
    Custom,
}

/// サンドボックスマネージャー
///
/// サンドボックス環境の作成と管理を行います。
pub struct SandboxManager {
    /// アクティブなサンドボックス
    active_sandboxes: Mutex<HashSet<String>>,
    /// デフォルトのセキュリティプロファイル
    default_profile: Mutex<SandboxSecurityProfile>,
}

impl SandboxManager {
    /// 新しいサンドボックスマネージャーを作成
    pub fn new() -> Self {
        Self {
            active_sandboxes: Mutex::new(HashSet::new()),
            default_profile: Mutex::new(SandboxSecurityProfile::Standard),
        }
    }

    /// デフォルトのセキュリティプロファイルを設定
    pub fn set_default_profile(&self, profile: SandboxSecurityProfile) {
        let mut default_profile = self.default_profile.lock().unwrap();
        *default_profile = profile;
    }

    /// デフォルトのセキュリティプロファイルを取得
    pub fn get_default_profile(&self) -> SandboxSecurityProfile {
        *self.default_profile.lock().unwrap()
    }

    /// サンドボックスを作成
    pub fn create_sandbox(&self, sandbox_id: &str, config: &SandboxConfig) -> Result<(), SandboxError> {
        if !config.enable_sandbox {
            return Err(SandboxError::SandboxNotEnabled);
        }
        
        let mut active_sandboxes = self.active_sandboxes.lock().unwrap();
        if active_sandboxes.contains(sandbox_id) {
            return Err(SandboxError::SandboxAlreadyExists);
        }
        
        // ここでサンドボックス環境を実際に作成する
        #[cfg(target_os = "linux")]
        {
            // Linuxでは、seccompやnamespaceを使用してサンドボックスを作成
            self.create_linux_sandbox(sandbox_id, config)?;
        }
        
        #[cfg(target_os = "macos")]
        {
            // macOSでは、App Sandboxingの仕組みを使用
            self.create_macos_sandbox(sandbox_id, config)?;
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windowsでは、App Containerの仕組みを使用
            self.create_windows_sandbox(sandbox_id, config)?;
        }
        
        active_sandboxes.insert(sandbox_id.to_string());
        Ok(())
    }

    /// サンドボックスを破棄
    pub fn destroy_sandbox(&self, sandbox_id: &str) -> Result<(), SandboxError> {
        let mut active_sandboxes = self.active_sandboxes.lock().unwrap();
        if !active_sandboxes.contains(sandbox_id) {
            return Err(SandboxError::SandboxNotFound);
        }
        
        // ここでサンドボックス環境を実際に破棄する
        #[cfg(target_os = "linux")]
        {
            // Linuxのサンドボックスを破棄
            self.destroy_linux_sandbox(sandbox_id)?;
        }
        
        #[cfg(target_os = "macos")]
        {
            // macOSのサンドボックスを破棄
            self.destroy_macos_sandbox(sandbox_id)?;
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windowsのサンドボックスを破棄
            self.destroy_windows_sandbox(sandbox_id)?;
        }
        
        active_sandboxes.remove(sandbox_id);
        Ok(())
    }

    /// サンドボックスが存在するかどうかを確認
    pub fn has_sandbox(&self, sandbox_id: &str) -> bool {
        let active_sandboxes = self.active_sandboxes.lock().unwrap();
        active_sandboxes.contains(sandbox_id)
    }

    /// アクティブなサンドボックスの数を取得
    pub fn count_active_sandboxes(&self) -> usize {
        let active_sandboxes = self.active_sandboxes.lock().unwrap();
        active_sandboxes.len()
    }

    /// アクティブなサンドボックスのIDリストを取得
    pub fn get_active_sandbox_ids(&self) -> Vec<String> {
        let active_sandboxes = self.active_sandboxes.lock().unwrap();
        active_sandboxes.iter().cloned().collect()
    }

    // 以下はプラットフォーム固有の実装
    
    #[cfg(target_os = "linux")]
    fn create_linux_sandbox(&self, sandbox_id: &str, config: &SandboxConfig) -> Result<(), SandboxError> {
        info!("Linuxサンドボックスを作成: {}", sandbox_id);
        
        // 実際のLinuxサンドボックス作成は、別途実装する必要がある
        // ここでは例として、成功したものとする
        
        Ok(())
    }
    
    #[cfg(target_os = "linux")]
    fn destroy_linux_sandbox(&self, sandbox_id: &str) -> Result<(), SandboxError> {
        info!("Linuxサンドボックスを破棄: {}", sandbox_id);
        
        // 実際のLinuxサンドボックス破棄は、別途実装する必要がある
        // ここでは例として、成功したものとする
        
        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    fn create_macos_sandbox(&self, sandbox_id: &str, config: &SandboxConfig) -> Result<(), SandboxError> {
        info!("macOSサンドボックスを作成: {}", sandbox_id);
        
        // 実際のmacOSサンドボックス作成は、別途実装する必要がある
        // ここでは例として、成功したものとする
        
        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    fn destroy_macos_sandbox(&self, sandbox_id: &str) -> Result<(), SandboxError> {
        info!("macOSサンドボックスを破棄: {}", sandbox_id);
        
        // 実際のmacOSサンドボックス破棄は、別途実装する必要がある
        // ここでは例として、成功したものとする
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn create_windows_sandbox(&self, sandbox_id: &str, config: &SandboxConfig) -> Result<(), SandboxError> {
        info!("Windowsサンドボックスを作成: {}", sandbox_id);
        
        // 実際のWindowsサンドボックス作成は、別途実装する必要がある
        // ここでは例として、成功したものとする
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn destroy_windows_sandbox(&self, sandbox_id: &str) -> Result<(), SandboxError> {
        info!("Windowsサンドボックスを破棄: {}", sandbox_id);
        
        // 実際のWindowsサンドボックス破棄は、別途実装する必要がある
        // ここでは例として、成功したものとする
        
        Ok(())
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

/// サンドボックスエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxError {
    /// サンドボックスが有効になっていない
    SandboxNotEnabled,
    /// サンドボックスがすでに存在する
    SandboxAlreadyExists,
    /// サンドボックスが見つからない
    SandboxNotFound,
    /// 権限がない
    PermissionDenied,
    /// システムエラー
    SystemError(String),
    /// リソース制限エラー
    ResourceLimitError(String),
    /// その他のエラー
    Other(String),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxError::SandboxNotEnabled => write!(f, "サンドボックスが有効になっていません"),
            SandboxError::SandboxAlreadyExists => write!(f, "サンドボックスはすでに存在します"),
            SandboxError::SandboxNotFound => write!(f, "サンドボックスが見つかりません"),
            SandboxError::PermissionDenied => write!(f, "サンドボックス操作の権限がありません"),
            SandboxError::SystemError(msg) => write!(f, "システムエラー: {}", msg),
            SandboxError::ResourceLimitError(msg) => write!(f, "リソース制限エラー: {}", msg),
            SandboxError::Other(msg) => write!(f, "その他のエラー: {}", msg),
        }
    }
}

impl std::error::Error for SandboxError {}

/// サンドボックスプロセス
///
/// サンドボックス内で実行されるプロセスを表します。
pub struct SandboxProcess {
    /// プロセスID
    pub pid: u32,
    /// サンドボックスID
    pub sandbox_id: String,
    /// プロセス名
    pub name: String,
    /// コマンドライン
    pub command_line: String,
    /// 開始時刻
    pub start_time: std::time::SystemTime,
    /// メモリ使用量（バイト）
    pub memory_usage: usize,
    /// CPU使用率（0-100）
    pub cpu_usage: f32,
}

impl SandboxProcess {
    /// 新しいサンドボックスプロセスを作成
    pub fn new(pid: u32, sandbox_id: &str, name: &str, command_line: &str) -> Self {
        Self {
            pid,
            sandbox_id: sandbox_id.to_string(),
            name: name.to_string(),
            command_line: command_line.to_string(),
            start_time: std::time::SystemTime::now(),
            memory_usage: 0,
            cpu_usage: 0.0,
        }
    }

    /// プロセス情報を更新
    pub fn update_stats(&mut self, memory_usage: usize, cpu_usage: f32) {
        self.memory_usage = memory_usage;
        self.cpu_usage = cpu_usage;
    }

    /// プロセスの実行時間を取得
    pub fn get_runtime(&self) -> std::time::Duration {
        std::time::SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
    }

    /// プロセスを終了
    pub fn terminate(&self) -> Result<(), SandboxError> {
        // 実際のプロセス終了処理は、プラットフォームに依存する
        #[cfg(unix)]
        {
            use std::process::Command;
            Command::new("kill")
                .arg(self.pid.to_string())
                .status()
                .map_err(|e| SandboxError::SystemError(e.to_string()))?;
        }
        
        #[cfg(windows)]
        {
            use std::process::Command;
            Command::new("taskkill")
                .args(&["/F", "/PID", &self.pid.to_string()])
                .status()
                .map_err(|e| SandboxError::SystemError(e.to_string()))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(!config.enable_sandbox);
        assert!(!config.restrict_network);
        assert!(!config.restrict_file_access);
        assert_eq!(config.allowed_paths.len(), 0);
    }

    #[test]
    fn test_sandbox_config_new() {
        let config = SandboxConfig::new(true);
        assert!(config.enable_sandbox);
        assert!(!config.restrict_network);
        assert!(!config.restrict_file_access);
    }

    #[test]
    fn test_sandbox_security_profile() {
        let mut config = SandboxConfig::new(true);
        
        // スタンダードプロファイルを適用
        config.apply_security_profile(SandboxSecurityProfile::Standard);
        assert!(config.restrict_network);
        assert!(config.restrict_file_access);
        assert!(config.restrict_devices);
        assert!(!config.restrict_ipc);
        assert!(!config.restrict_graphics);
        assert_eq!(config.allowed_ports.len(), 4);
        
        // 厳格なプロファイルを適用
        config.apply_security_profile(SandboxSecurityProfile::Strict);
        assert!(config.restrict_network);
        assert!(config.restrict_file_access);
        assert!(config.restrict_devices);
        assert!(config.restrict_ipc);
        assert!(config.restrict_graphics);
        assert!(config.enable_syscall_filtering);
        assert!(config.enable_capabilities_filtering);
    }

    #[test]
    fn test_is_host_allowed() {
        let mut config = SandboxConfig::new(true);
        config.restrict_network = true;
        config.allowed_hosts = vec![
            "example.com".to_string(),
            "*.trusted.org".to_string(),
            "api.*".to_string(),
        ];
        
        assert!(config.is_host_allowed("example.com"));
        assert!(config.is_host_allowed("server.trusted.org"));
        assert!(config.is_host_allowed("api.example.com"));
        assert!(!config.is_host_allowed("malicious.com"));
    }

    #[test]
    fn test_is_port_allowed() {
        let mut config = SandboxConfig::new(true);
        config.restrict_network = true;
        config.allowed_ports = vec![80, 443, 8080];
        
        assert!(config.is_port_allowed(80));
        assert!(config.is_port_allowed(443));
        assert!(config.is_port_allowed(8080));
        assert!(!config.is_port_allowed(22));
        assert!(!config.is_port_allowed(3306));
    }

    #[test]
    fn test_sandbox_manager() {
        let manager = SandboxManager::new();
        assert_eq!(manager.get_default_profile(), SandboxSecurityProfile::Standard);
        
        manager.set_default_profile(SandboxSecurityProfile::Strict);
        assert_eq!(manager.get_default_profile(), SandboxSecurityProfile::Strict);
        
        assert_eq!(manager.count_active_sandboxes(), 0);
    }

    #[test]
    fn test_sandbox_process() {
        let process = SandboxProcess::new(1234, "test-sandbox", "test-app", "/usr/bin/test-app");
        assert_eq!(process.pid, 1234);
        assert_eq!(process.sandbox_id, "test-sandbox");
        assert_eq!(process.name, "test-app");
        assert_eq!(process.command_line, "/usr/bin/test-app");
        assert_eq!(process.memory_usage, 0);
        assert_eq!(process.cpu_usage, 0.0);
        
        // 統計を更新
        let mut process = process;
        process.update_stats(1024 * 1024, 10.5);
        assert_eq!(process.memory_usage, 1024 * 1024);
        assert_eq!(process.cpu_usage, 10.5);
    }
} 