// LumosDesktop Security Policy Module
//
// このモジュールはセキュリティポリシーを管理します。
// ポリシーの定義、ポリシーの評価、ポリシーマネージャーなどを提供します。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

use crate::core::system::logging;
use super::permission::{Permission, PermissionSet};
use super::SecurityLevel;

/// セキュリティポリシーのタイプ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyType {
    /// 許可ポリシー
    Allow,
    /// 拒否ポリシー
    Deny,
    /// プロンプトポリシー（ユーザーに確認）
    Prompt,
    /// 条件付きポリシー
    Conditional,
}

/// セキュリティポリシーのターゲット
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyTarget {
    /// ファイルパスに基づくポリシー
    FilePath(String),
    /// ネットワークアドレスに基づくポリシー
    NetworkAddress(String),
    /// プロセス名に基づくポリシー
    ProcessName(String),
    /// デバイスIDに基づくポリシー
    DeviceId(String),
    /// アプリケーションIDに基づくポリシー
    AppId(String),
    /// すべてのターゲットに適用
    All,
}

/// セキュリティポリシーの条件
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyCondition {
    /// ユーザーIDに基づく条件
    UserId(String),
    /// 時間帯に基づく条件
    TimeRange {
        start_hour: u8,
        start_minute: u8,
        end_hour: u8,
        end_minute: u8,
    },
    /// ネットワーク接続タイプに基づく条件
    NetworkType(String),
    /// 位置情報に基づく条件
    Location {
        latitude: f64,
        longitude: f64,
        radius: f64,
    },
    /// システム負荷に基づく条件
    SystemLoad(f64),
    /// バッテリーレベルに基づく条件
    BatteryLevel(u8),
    /// 複数条件の論理AND
    And(Vec<PolicyCondition>),
    /// 複数条件の論理OR
    Or(Vec<PolicyCondition>),
    /// 条件の論理NOT
    Not(Box<PolicyCondition>),
}

/// セキュリティポリシー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// ポリシーID
    pub id: String,
    /// ポリシー名
    pub name: String,
    /// ポリシーの説明
    pub description: String,
    /// ポリシーのタイプ
    pub policy_type: PolicyType,
    /// 対象の権限
    pub permission: Permission,
    /// ポリシーのターゲット
    pub target: PolicyTarget,
    /// ポリシーの条件（条件付きポリシーの場合）
    pub condition: Option<PolicyCondition>,
    /// ポリシーの優先度（高いほど優先）
    pub priority: u32,
    /// ポリシーが有効かどうか
    pub enabled: bool,
    /// ポリシーの作成者
    pub creator: String,
    /// ポリシーの作成日時
    pub created_at: i64,
    /// ポリシーの更新日時
    pub updated_at: i64,
}

impl SecurityPolicy {
    /// 新しいセキュリティポリシーを作成
    pub fn new(
        id: String,
        name: String,
        description: String,
        policy_type: PolicyType,
        permission: Permission,
        target: PolicyTarget,
        priority: u32,
        creator: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        Self {
            id,
            name,
            description,
            policy_type,
            permission,
            target,
            condition: None,
            priority,
            enabled: true,
            creator,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// 条件付きポリシーを作成
    pub fn with_condition(mut self, condition: PolicyCondition) -> Self {
        self.condition = Some(condition);
        self
    }
    
    /// ポリシーを有効化
    pub fn enable(&mut self) {
        self.enabled = true;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        self.updated_at = now;
    }
    
    /// ポリシーを無効化
    pub fn disable(&mut self) {
        self.enabled = false;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        self.updated_at = now;
    }
    
    /// ポリシーが適用可能かどうかを確認
    pub fn is_applicable(&self, perm: &Permission, target: &PolicyTarget) -> bool {
        // ポリシーが無効の場合は適用不可
        if !self.enabled {
            return false;
        }
        
        // 権限が一致しない場合は適用不可
        if *perm != self.permission && self.permission != Permission::Custom("*".to_string()) {
            return false;
        }
        
        // ターゲットが一致するか、ポリシーのターゲットがAllの場合は適用可能
        self.target == *target || self.target == PolicyTarget::All
    }
    
    /// ポリシーの条件を評価
    pub fn evaluate_condition(&self, context: &PolicyEvaluationContext) -> bool {
        match &self.condition {
            Some(cond) => Self::evaluate_condition_internal(cond, context),
            None => true,
        }
    }
    
    /// 内部条件評価ロジック
    fn evaluate_condition_internal(
        condition: &PolicyCondition,
        context: &PolicyEvaluationContext
    ) -> bool {
        match condition {
            PolicyCondition::UserId(user_id) => {
                context.user_id.as_ref().map_or(false, |id| id == user_id)
            }
            
            PolicyCondition::TimeRange { start_hour, start_minute, end_hour, end_minute } => {
                if let Some(now) = context.current_time {
                    let start_minutes = *start_hour as u32 * 60 + *start_minute as u32;
                    let end_minutes = *end_hour as u32 * 60 + *end_minute as u32;
                    let now_minutes = now.tm_hour as u32 * 60 + now.tm_min as u32;
                    
                    if start_minutes <= end_minutes {
                        // 同じ日の時間範囲
                        start_minutes <= now_minutes && now_minutes <= end_minutes
                    } else {
                        // 日をまたぐ時間範囲
                        start_minutes <= now_minutes || now_minutes <= end_minutes
                    }
                } else {
                    false
                }
            }
            
            PolicyCondition::NetworkType(network_type) => {
                context.network_type.as_ref().map_or(false, |nt| nt == network_type)
            }
            
            PolicyCondition::Location { latitude, longitude, radius } => {
                if let Some((lat, lon)) = context.location {
                    // 簡易的な距離計算（実際の実装では正確な地球上の距離計算を使用すべき）
                    let dlat = lat - latitude;
                    let dlon = lon - longitude;
                    let distance = (dlat * dlat + dlon * dlon).sqrt();
                    distance <= *radius
                } else {
                    false
                }
            }
            
            PolicyCondition::SystemLoad(threshold) => {
                context.system_load.map_or(false, |load| load <= *threshold)
            }
            
            PolicyCondition::BatteryLevel(min_level) => {
                context.battery_level.map_or(false, |level| level >= *min_level)
            }
            
            PolicyCondition::And(conditions) => {
                conditions.iter().all(|c| Self::evaluate_condition_internal(c, context))
            }
            
            PolicyCondition::Or(conditions) => {
                conditions.iter().any(|c| Self::evaluate_condition_internal(c, context))
            }
            
            PolicyCondition::Not(condition) => {
                !Self::evaluate_condition_internal(condition, context)
            }
        }
    }
}

/// ポリシー評価コンテキスト
#[derive(Debug, Clone)]
pub struct PolicyEvaluationContext {
    /// ユーザーID
    pub user_id: Option<String>,
    /// 現在時刻
    pub current_time: Option<libc::tm>,
    /// ネットワークタイプ
    pub network_type: Option<String>,
    /// 位置情報 (緯度, 経度)
    pub location: Option<(f64, f64)>,
    /// システム負荷
    pub system_load: Option<f64>,
    /// バッテリーレベル (0-100)
    pub battery_level: Option<u8>,
    /// カスタムコンテキスト
    pub custom: HashMap<String, String>,
}

impl PolicyEvaluationContext {
    /// 新しいポリシー評価コンテキストを作成
    pub fn new() -> Self {
        Self {
            user_id: None,
            current_time: None,
            network_type: None,
            location: None,
            system_load: None,
            battery_level: None,
            custom: HashMap::new(),
        }
    }
    
    /// ユーザーIDを設定
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    /// 現在時刻を設定
    pub fn with_current_time(mut self) -> Self {
        let mut now: libc::time_t = 0;
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        
        unsafe {
            libc::time(&mut now);
            libc::localtime_r(&now, &mut tm);
        }
        
        self.current_time = Some(tm);
        self
    }
    
    /// ネットワークタイプを設定
    pub fn with_network_type(mut self, network_type: String) -> Self {
        self.network_type = Some(network_type);
        self
    }
    
    /// 位置情報を設定
    pub fn with_location(mut self, latitude: f64, longitude: f64) -> Self {
        self.location = Some((latitude, longitude));
        self
    }
    
    /// システム負荷を設定
    pub fn with_system_load(mut self, load: f64) -> Self {
        self.system_load = Some(load);
        self
    }
    
    /// バッテリーレベルを設定
    pub fn with_battery_level(mut self, level: u8) -> Self {
        self.battery_level = Some(level.min(100));
        self
    }
    
    /// カスタムコンテキストを追加
    pub fn with_custom(mut self, key: String, value: String) -> Self {
        self.custom.insert(key, value);
        self
    }
}

/// ポリシーマネージャー
pub struct PolicyManager {
    /// ポリシーストア
    policies: RwLock<Vec<SecurityPolicy>>,
    /// ポリシー設定ファイルのパス
    config_path: Option<PathBuf>,
}

impl PolicyManager {
    /// 新しいポリシーマネージャーを作成
    pub fn new() -> Self {
        let logger = logging::get_logger("policy_manager");
        logging::debug!(logger, "PolicyManagerを初期化中...");
        
        Self {
            policies: RwLock::new(Vec::new()),
            config_path: None,
        }
    }
    
    /// 新しいポリシーマネージャーを設定ファイルから作成
    pub fn with_config(config_path: PathBuf) -> Self {
        let logger = logging::get_logger("policy_manager");
        logging::debug!(logger, "PolicyManagerを設定ファイルから初期化中...");
        
        let manager = Self {
            policies: RwLock::new(Vec::new()),
            config_path: Some(config_path.clone()),
        };
        
        if let Err(e) = manager.load_policies() {
            logging::error!(logger, "ポリシーのロードに失敗しました: {}", e);
        }
        
        manager
    }
    
    /// ポリシーをファイルからロード
    pub fn load_policies(&self) -> Result<(), String> {
        let logger = logging::get_logger("policy_manager");
        
        let config_path = self.config_path.as_ref().ok_or_else(|| {
            "設定ファイルパスが設定されていません".to_string()
        })?;
        
        logging::info!(logger, "ポリシーをロード中: {:?}", config_path);
        
        // ファイルが存在しない場合は空のポリシーセットを作成して終了
        if !config_path.exists() {
            logging::warn!(logger, "ポリシー設定ファイルが見つかりません: {:?}", config_path);
            return Ok(());
        }
        
        // ファイルからポリシーを読み込む
        let file_content = std::fs::read_to_string(config_path).map_err(|e| {
            format!("ポリシー設定ファイルの読み込みに失敗しました: {}", e)
        })?;
        
        let policies: Vec<SecurityPolicy> = serde_json::from_str(&file_content).map_err(|e| {
            format!("ポリシー設定のパースに失敗しました: {}", e)
        })?;
        
        // ポリシーをストアに格納
        let mut policies_lock = self.policies.write().map_err(|_| {
            "ポリシーストアへのアクセス中にエラーが発生しました".to_string()
        })?;
        
        *policies_lock = policies;
        
        logging::info!(logger, "{}個のポリシーをロードしました", policies_lock.len());
        
        Ok(())
    }
    
    /// ポリシーをファイルに保存
    pub fn save_policies(&self) -> Result<(), String> {
        let logger = logging::get_logger("policy_manager");
        
        let config_path = self.config_path.as_ref().ok_or_else(|| {
            "設定ファイルパスが設定されていません".to_string()
        })?;
        
        logging::info!(logger, "ポリシーを保存中: {:?}", config_path);
        
        // ポリシーをシリアライズ
        let policies_lock = self.policies.read().map_err(|_| {
            "ポリシーストアへのアクセス中にエラーが発生しました".to_string()
        })?;
        
        let json = serde_json::to_string_pretty(&*policies_lock).map_err(|e| {
            format!("ポリシーのシリアライズに失敗しました: {}", e)
        })?;
        
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!("ディレクトリの作成に失敗しました: {}", e)
                })?;
            }
        }
        
        // ファイルに書き込み
        std::fs::write(config_path, json).map_err(|e| {
            format!("ポリシー設定ファイルの書き込みに失敗しました: {}", e)
        })?;
        
        logging::info!(logger, "{}個のポリシーを保存しました", policies_lock.len());
        
        Ok(())
    }
    
    /// ポリシーを追加
    pub fn add_policy(&self, policy: SecurityPolicy) -> Result<(), String> {
        let mut policies_lock = self.policies.write().map_err(|_| {
            "ポリシーストアへのアクセス中にエラーが発生しました".to_string()
        })?;
        
        // 同じIDのポリシーがあれば更新
        for i in 0..policies_lock.len() {
            if policies_lock[i].id == policy.id {
                policies_lock[i] = policy;
                return Ok(());
            }
        }
        
        // 新規ポリシーを追加
        policies_lock.push(policy);
        
        // 優先度順にソート
        policies_lock.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(())
    }
    
    /// ポリシーを削除
    pub fn remove_policy(&self, policy_id: &str) -> Result<(), String> {
        let mut policies_lock = self.policies.write().map_err(|_| {
            "ポリシーストアへのアクセス中にエラーが発生しました".to_string()
        })?;
        
        let initial_len = policies_lock.len();
        policies_lock.retain(|p| p.id != policy_id);
        
        if policies_lock.len() == initial_len {
            return Err(format!("ポリシー '{}'が見つかりません", policy_id));
        }
        
        Ok(())
    }
    
    /// ポリシーを取得
    pub fn get_policy(&self, policy_id: &str) -> Result<SecurityPolicy, String> {
        let policies_lock = self.policies.read().map_err(|_| {
            "ポリシーストアへのアクセス中にエラーが発生しました".to_string()
        })?;
        
        for policy in policies_lock.iter() {
            if policy.id == policy_id {
                return Ok(policy.clone());
            }
        }
        
        Err(format!("ポリシー '{}'が見つかりません", policy_id))
    }
    
    /// すべてのポリシーを取得
    pub fn get_all_policies(&self) -> Result<Vec<SecurityPolicy>, String> {
        let policies_lock = self.policies.read().map_err(|_| {
            "ポリシーストアへのアクセス中にエラーが発生しました".to_string()
        })?;
        
        Ok(policies_lock.clone())
    }
    
    /// 権限とターゲットに適用可能なポリシーを取得
    pub fn get_applicable_policies(
        &self,
        permission: &Permission,
        target: &PolicyTarget
    ) -> Result<Vec<SecurityPolicy>, String> {
        let policies_lock = self.policies.read().map_err(|_| {
            "ポリシーストアへのアクセス中にエラーが発生しました".to_string()
        })?;
        
        let applicable = policies_lock
            .iter()
            .filter(|p| p.is_applicable(permission, target))
            .cloned()
            .collect();
        
        Ok(applicable)
    }
    
    /// ポリシーを評価し、アクションが許可されるかどうかを決定
    pub fn evaluate(
        &self,
        permission: &Permission,
        target: &PolicyTarget,
        context: &PolicyEvaluationContext
    ) -> Result<PolicyType, String> {
        let logger = logging::get_logger("policy_manager");
        
        let applicable_policies = self.get_applicable_policies(permission, target)?;
        
        if applicable_policies.is_empty() {
            logging::debug!(
                logger, 
                "適用可能なポリシーが見つからないため、デフォルト動作(Prompt)を返します"
            );
            return Ok(PolicyType::Prompt);
        }
        
        // 優先度順に評価（優先度の高いポリシーが先に評価される）
        for policy in applicable_policies {
            // 条件付きポリシーの場合は条件を評価
            if policy.condition.is_some() && !policy.evaluate_condition(context) {
                logging::debug!(
                    logger,
                    "ポリシー '{}' の条件が満たされなかったため、スキップします",
                    policy.id
                );
                continue;
            }
            
            logging::debug!(
                logger,
                "ポリシー '{}' を適用します: {:?}",
                policy.id,
                policy.policy_type
            );
            
            return Ok(policy.policy_type.clone());
        }
        
        // 適用可能なポリシーがない場合は確認を求める
        logging::debug!(
            logger,
            "条件を満たすポリシーが見つからないため、デフォルト動作(Prompt)を返します"
        );
        
        Ok(PolicyType::Prompt)
    }
}

// テスト
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_policy_creation() {
        let policy = SecurityPolicy::new(
            "test1".to_string(),
            "テストポリシー".to_string(),
            "これはテストポリシーです".to_string(),
            PolicyType::Allow,
            Permission::FileRead,
            PolicyTarget::All,
            100,
            "admin".to_string(),
        );
        
        assert_eq!(policy.id, "test1");
        assert_eq!(policy.name, "テストポリシー");
        assert_eq!(policy.policy_type, PolicyType::Allow);
        assert_eq!(policy.priority, 100);
        assert_eq!(policy.enabled, true);
    }
    
    #[test]
    fn test_policy_condition() {
        let condition = PolicyCondition::And(vec![
            PolicyCondition::TimeRange {
                start_hour: 9,
                start_minute: 0,
                end_hour: 17,
                end_minute: 0,
            },
            PolicyCondition::BatteryLevel(20),
        ]);
        
        // 条件付きポリシーを作成
        let policy = SecurityPolicy::new(
            "test2".to_string(),
            "条件付きポリシー".to_string(),
            "これは条件付きポリシーです".to_string(),
            PolicyType::Allow,
            Permission::FileRead,
            PolicyTarget::All,
            100,
            "admin".to_string(),
        ).with_condition(condition);
        
        assert!(policy.condition.is_some());
    }
    
    #[test]
    fn test_policy_manager() {
        let manager = PolicyManager::new();
        
        // ポリシーを追加
        let policy1 = SecurityPolicy::new(
            "p1".to_string(),
            "ポリシー1".to_string(),
            "これはポリシー1です".to_string(),
            PolicyType::Allow,
            Permission::FileRead,
            PolicyTarget::All,
            100,
            "admin".to_string(),
        );
        
        let policy2 = SecurityPolicy::new(
            "p2".to_string(),
            "ポリシー2".to_string(),
            "これはポリシー2です".to_string(),
            PolicyType::Deny,
            Permission::FileWrite,
            PolicyTarget::FilePath("/etc/*".to_string()),
            200,
            "admin".to_string(),
        );
        
        assert!(manager.add_policy(policy1).is_ok());
        assert!(manager.add_policy(policy2).is_ok());
        
        // ポリシーを取得
        let p1 = manager.get_policy("p1").unwrap();
        assert_eq!(p1.id, "p1");
        
        // すべてのポリシーを取得
        let all = manager.get_all_policies().unwrap();
        assert_eq!(all.len(), 2);
        
        // 特定の条件に一致するポリシーを取得
        let applicable = manager.get_applicable_policies(
            &Permission::FileRead,
            &PolicyTarget::All,
        ).unwrap();
        assert_eq!(applicable.len(), 1);
        assert_eq!(applicable[0].id, "p1");
        
        // ポリシーを評価
        let context = PolicyEvaluationContext::new();
        let result = manager.evaluate(
            &Permission::FileRead,
            &PolicyTarget::All,
            &context,
        ).unwrap();
        assert_eq!(result, PolicyType::Allow);
        
        // ポリシーを削除
        assert!(manager.remove_policy("p1").is_ok());
        assert!(manager.get_policy("p1").is_err());
    }
} 