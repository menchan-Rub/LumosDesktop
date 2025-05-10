// LumosDesktop リソースアラートモジュール
// ハードウェアリソースの問題を検出して通知します

use std::fmt;
use std::time::{Duration, Instant};

/// アラートの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertType {
    /// 高いCPU使用率
    HighCpuUsage,
    /// 高いメモリ使用率
    HighMemoryUsage,
    /// 高いディスク使用率
    HighDiskUsage,
    /// 高い温度
    HighTemperature,
    /// 低ディスク空き容量
    LowDiskSpace,
    /// 低バッテリー残量
    LowBattery,
    /// 危機的なバッテリー残量
    CriticalBattery,
    /// ディスク健康状態の問題
    StorageHealth,
    /// 電源供給元の変更
    PowerSourceChanged,
    /// センサー異常
    SensorAnomaly,
    /// スワップメモリの過度な使用
    HighSwapUsage,
    /// プロセスのメモリリーク
    MemoryLeak,
    /// 高いネットワーク使用率
    HighNetworkUsage,
    /// ハードウェアエラー
    HardwareError,
    /// その他
    Other,
}

impl fmt::Display for AlertType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertType::HighCpuUsage => write!(f, "高CPU使用率"),
            AlertType::HighMemoryUsage => write!(f, "高メモリ使用率"),
            AlertType::HighDiskUsage => write!(f, "高ディスク使用率"),
            AlertType::HighTemperature => write!(f, "高温警告"),
            AlertType::LowDiskSpace => write!(f, "ディスク空き容量不足"),
            AlertType::LowBattery => write!(f, "バッテリー残量低下"),
            AlertType::CriticalBattery => write!(f, "バッテリー残量危機"),
            AlertType::StorageHealth => write!(f, "ストレージ健康状態異常"),
            AlertType::PowerSourceChanged => write!(f, "電源状態変更"),
            AlertType::SensorAnomaly => write!(f, "センサー異常"),
            AlertType::HighSwapUsage => write!(f, "高スワップ使用率"),
            AlertType::MemoryLeak => write!(f, "メモリリーク検出"),
            AlertType::HighNetworkUsage => write!(f, "高ネットワーク使用率"),
            AlertType::HardwareError => write!(f, "ハードウェアエラー"),
            AlertType::Other => write!(f, "その他のアラート"),
        }
    }
}

/// アラートの重要度レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AlertLevel {
    /// 情報提供のみ
    Info,
    /// 注意が必要
    Warning,
    /// 重大な問題
    Critical,
    /// 緊急の問題（即時対応が必要）
    Emergency,
}

impl fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertLevel::Info => write!(f, "情報"),
            AlertLevel::Warning => write!(f, "警告"),
            AlertLevel::Critical => write!(f, "重大"),
            AlertLevel::Emergency => write!(f, "緊急"),
        }
    }
}

/// リソースアラート
#[derive(Debug, Clone)]
pub struct ResourceAlert {
    /// アラートの種類
    pub alert_type: AlertType,
    /// アラートの重要度レベル
    pub level: AlertLevel,
    /// アラートの詳細メッセージ
    pub message: String,
    /// 問題のリソース識別子（デバイス名など）
    pub resource_id: Option<String>,
    /// 計測値（該当する場合）
    pub measured_value: Option<f64>,
    /// 閾値（該当する場合）
    pub threshold: Option<f64>,
    /// 単位（該当する場合）
    pub unit: Option<String>,
    /// アラート発生時刻
    pub timestamp: Instant,
    /// 推奨アクション
    pub recommended_action: Option<String>,
    /// アラートが解決されたかどうか
    pub is_resolved: bool,
    /// 解決時刻（解決された場合）
    pub resolved_at: Option<Instant>,
    /// アラートの一意識別子
    pub id: String,
}

impl ResourceAlert {
    /// 基本的なリソースアラートを作成
    pub fn new(alert_type: AlertType, level: AlertLevel, message: String) -> Self {
        // ランダムなIDを生成
        let id = format!("alert-{}-{}", u64::from_be_bytes(Instant::now().elapsed().as_secs().to_be_bytes()), fastrand::u32(..));
        
        Self {
            alert_type,
            level,
            message,
            resource_id: None,
            measured_value: None,
            threshold: None,
            unit: None,
            timestamp: Instant::now(),
            recommended_action: None,
            is_resolved: false,
            resolved_at: None,
            id,
        }
    }
    
    /// リソース識別子を設定
    pub fn with_resource_id(mut self, resource_id: String) -> Self {
        self.resource_id = Some(resource_id);
        self
    }
    
    /// 計測値を設定
    pub fn with_value(mut self, value: f64, unit: Option<String>) -> Self {
        self.measured_value = Some(value);
        self.unit = unit;
        self
    }
    
    /// 閾値を設定
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = Some(threshold);
        self
    }
    
    /// 推奨アクションを設定
    pub fn with_recommended_action(mut self, action: String) -> Self {
        self.recommended_action = Some(action);
        self
    }
    
    /// アラートを解決済みとしてマーク
    pub fn resolve(&mut self) {
        self.is_resolved = true;
        self.resolved_at = Some(Instant::now());
    }
    
    /// アラート発生からの経過時間を取得
    pub fn duration_since_alert(&self) -> Duration {
        self.timestamp.elapsed()
    }
    
    /// アラート解決からの経過時間を取得（解決済みの場合）
    pub fn duration_since_resolution(&self) -> Option<Duration> {
        self.resolved_at.map(|t| t.elapsed())
    }
    
    /// アラート解決までにかかった時間を取得（解決済みの場合）
    pub fn time_to_resolution(&self) -> Option<Duration> {
        self.resolved_at.map(|resolved| {
            if resolved > self.timestamp {
                resolved.duration_since(self.timestamp)
            } else {
                Duration::from_secs(0)
            }
        })
    }
    
    /// アラートの重要性に基づいて通知すべきかどうかを判断
    pub fn should_notify(&self) -> bool {
        // 情報レベルのアラートは通知しない例外パターン
        if self.level == AlertLevel::Info {
            match self.alert_type {
                // 電源変更は通知する価値がある
                AlertType::PowerSourceChanged => true,
                // 他の「情報」アラートは通常通知しない
                _ => false,
            }
        } else {
            // 警告、重大、緊急レベルはすべて通知
            true
        }
    }
    
    /// 同じタイプのアラートを更新すべきかどうかを判断
    pub fn should_update(&self, previous: &ResourceAlert) -> bool {
        if self.alert_type != previous.alert_type {
            return false;
        }
        
        // 同じリソースに関するアラートかどうか
        if self.resource_id != previous.resource_id {
            return false;
        }
        
        // レベルの変化があれば更新すべき
        if self.level != previous.level {
            return true;
        }
        
        // 解決状態の変化があれば更新すべき
        if self.is_resolved != previous.is_resolved {
            return true;
        }
        
        // 値が大幅に変化した場合も更新すべき
        if let (Some(current), Some(prev)) = (self.measured_value, previous.measured_value) {
            let change_percent = ((current - prev) / prev).abs() * 100.0;
            if change_percent > 20.0 {  // 20%以上の変化を重要と判断
                return true;
            }
        }
        
        // 最後の更新から一定時間以上経過していれば更新すべき
        previous.timestamp.elapsed() > Duration::from_secs(300)  // 5分
    }
    
    /// リソースアラートの完全な説明を生成
    pub fn full_description(&self) -> String {
        let mut description = format!("[{}] {}: {}", self.level, self.alert_type, self.message);
        
        if let Some(resource) = &self.resource_id {
            description.push_str(&format!("\nリソース: {}", resource));
        }
        
        if let Some(value) = self.measured_value {
            if let Some(unit) = &self.unit {
                description.push_str(&format!("\n計測値: {} {}", value, unit));
            } else {
                description.push_str(&format!("\n計測値: {}", value));
            }
            
            if let Some(threshold) = self.threshold {
                description.push_str(&format!(" (閾値: {})", threshold));
            }
        }
        
        if let Some(action) = &self.recommended_action {
            description.push_str(&format!("\n推奨アクション: {}", action));
        }
        
        if self.is_resolved {
            if let Some(resolved_at) = self.resolved_at {
                let resolution_time = resolved_at.duration_since(self.timestamp);
                description.push_str(&format!("\n解決済み: {} 秒後", resolution_time.as_secs()));
            } else {
                description.push_str("\n解決済み");
            }
        } else {
            let duration = self.timestamp.elapsed();
            description.push_str(&format!("\n未解決: {} 秒経過", duration.as_secs()));
        }
        
        description
    }
}

impl fmt::Display for ResourceAlert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.level, self.alert_type, self.message)
    }
}

/// 同種のアラートをグループ化するためのキー
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlertGroupKey {
    /// アラートの種類
    pub alert_type: AlertType,
    /// リソース識別子（存在する場合）
    pub resource_id: Option<String>,
}

impl From<&ResourceAlert> for AlertGroupKey {
    fn from(alert: &ResourceAlert) -> Self {
        Self {
            alert_type: alert.alert_type,
            resource_id: alert.resource_id.clone(),
        }
    }
}

/// 複数のアラートを管理するリソースアラートマネージャー
pub struct AlertManager {
    /// 現在のアクティブなアラート
    active_alerts: Vec<ResourceAlert>,
    /// 解決済みアラートの履歴
    alert_history: Vec<ResourceAlert>,
    /// 履歴保持の最大項目数
    max_history_items: usize,
}

impl AlertManager {
    /// 新しいアラートマネージャーを作成
    pub fn new() -> Self {
        Self {
            active_alerts: Vec::new(),
            alert_history: Vec::new(),
            max_history_items: 1000,
        }
    }
    
    /// 最大履歴保持数を設定
    pub fn with_max_history(mut self, max_items: usize) -> Self {
        self.max_history_items = max_items;
        self
    }
    
    /// 新しいアラートを追加または更新
    pub fn add_alert(&mut self, alert: ResourceAlert) -> &ResourceAlert {
        // 同じタイプの既存アラートを探す
        let key = AlertGroupKey::from(&alert);
        let existing_index = self.active_alerts.iter().position(|a| {
            AlertGroupKey::from(a) == key
        });
        
        if let Some(index) = existing_index {
            let existing = &mut self.active_alerts[index];
            
            // 既存のアラートを新しいアラートで更新すべきか判断
            if alert.should_update(existing) {
                *existing = alert;
            }
            
            &self.active_alerts[index]
        } else {
            // 新しいアラートを追加
            self.active_alerts.push(alert);
            self.active_alerts.last().unwrap()
        }
    }
    
    /// アラートを解決済みとしてマーク
    pub fn resolve_alert(&mut self, alert_id: &str) -> Option<&ResourceAlert> {
        if let Some(index) = self.active_alerts.iter().position(|a| a.id == alert_id) {
            let alert = &mut self.active_alerts[index];
            alert.resolve();
            
            // 履歴に追加
            let resolved_alert = alert.clone();
            self.alert_history.push(resolved_alert);
            
            // 履歴が最大数を超えた場合、古いものから削除
            if self.alert_history.len() > self.max_history_items {
                self.alert_history.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                self.alert_history.drain(0..self.alert_history.len() - self.max_history_items);
            }
            
            // アクティブリストから削除
            self.active_alerts.remove(index);
            
            // 履歴から最新の同じアラートを返す
            self.alert_history.iter()
                .filter(|a| a.id == alert_id)
                .last()
        } else {
            None
        }
    }
    
    /// 特定のタイプのアラートを解決済みとしてマーク
    pub fn resolve_alerts_by_type(&mut self, alert_type: AlertType, resource_id: Option<String>) -> Vec<ResourceAlert> {
        let key = AlertGroupKey {
            alert_type,
            resource_id,
        };
        
        // 解決すべきアラートのインデックスを収集
        let indices: Vec<usize> = self.active_alerts.iter()
            .enumerate()
            .filter(|(_, a)| AlertGroupKey::from(a) == key)
            .map(|(i, _)| i)
            .collect();
        
        // 解決したアラートを保存
        let mut resolved = Vec::new();
        
        // インデックスが大きい順に処理（削除による影響を避けるため）
        for i in indices.iter().rev() {
            let mut alert = self.active_alerts.remove(*i);
            alert.resolve();
            resolved.push(alert.clone());
            self.alert_history.push(alert);
        }
        
        // 履歴が最大数を超えた場合、古いものから削除
        if self.alert_history.len() > self.max_history_items {
            self.alert_history.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            self.alert_history.drain(0..self.alert_history.len() - self.max_history_items);
        }
        
        resolved
    }
    
    /// 全てのアクティブなアラートを取得
    pub fn get_active_alerts(&self) -> &[ResourceAlert] {
        &self.active_alerts
    }
    
    /// 特定の重要度レベル以上のアクティブなアラートを取得
    pub fn get_alerts_by_level(&self, min_level: AlertLevel) -> Vec<&ResourceAlert> {
        self.active_alerts.iter()
            .filter(|a| a.level >= min_level)
            .collect()
    }
    
    /// 特定のタイプのアクティブなアラートを取得
    pub fn get_alerts_by_type(&self, alert_type: AlertType) -> Vec<&ResourceAlert> {
        self.active_alerts.iter()
            .filter(|a| a.alert_type == alert_type)
            .collect()
    }
    
    /// 特定のリソースに関するアクティブなアラートを取得
    pub fn get_alerts_by_resource(&self, resource_id: &str) -> Vec<&ResourceAlert> {
        self.active_alerts.iter()
            .filter(|a| a.resource_id.as_ref().map_or(false, |id| id == resource_id))
            .collect()
    }
    
    /// 特定期間内のアラート履歴を取得
    pub fn get_alert_history(&self, duration: Duration) -> Vec<&ResourceAlert> {
        let cutoff = Instant::now() - duration;
        self.alert_history.iter()
            .filter(|a| a.timestamp >= cutoff)
            .collect()
    }
    
    /// 現在のアクティブアラート数を取得
    pub fn active_alerts_count(&self) -> usize {
        self.active_alerts.len()
    }
    
    /// 履歴アラート数を取得
    pub fn history_alerts_count(&self) -> usize {
        self.alert_history.len()
    }
    
    /// 最も重要なアラートを取得
    pub fn most_critical_alert(&self) -> Option<&ResourceAlert> {
        self.active_alerts.iter()
            .max_by_key(|a| a.level)
    }
    
    /// マネージャーをクリア
    pub fn clear(&mut self) {
        self.active_alerts.clear();
        self.alert_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_resource_alert_creation() {
        let alert = ResourceAlert::new(
            AlertType::HighCpuUsage,
            AlertLevel::Warning,
            "CPU使用率が90%を超えています".to_string()
        );
        
        assert_eq!(alert.alert_type, AlertType::HighCpuUsage);
        assert_eq!(alert.level, AlertLevel::Warning);
        assert_eq!(alert.message, "CPU使用率が90%を超えています");
        assert_eq!(alert.is_resolved, false);
        assert!(alert.resolved_at.is_none());
    }
    
    #[test]
    fn test_alert_resolution() {
        let mut alert = ResourceAlert::new(
            AlertType::HighCpuUsage,
            AlertLevel::Warning,
            "CPU使用率が90%を超えています".to_string()
        );
        
        assert_eq!(alert.is_resolved, false);
        
        // 少し遅延
        thread::sleep(Duration::from_millis(10));
        
        // アラートを解決
        alert.resolve();
        
        assert_eq!(alert.is_resolved, true);
        assert!(alert.resolved_at.is_some());
        
        // 解決までの時間が正しいか
        if let Some(time_to_resolution) = alert.time_to_resolution() {
            assert!(time_to_resolution.as_millis() >= 10);
        } else {
            panic!("time_to_resolution should return Some");
        }
    }
    
    #[test]
    fn test_alert_manager() {
        let mut manager = AlertManager::new();
        
        // アラートを追加
        let alert1 = ResourceAlert::new(
            AlertType::HighCpuUsage,
            AlertLevel::Warning,
            "CPU使用率が90%を超えています".to_string()
        ).with_resource_id("cpu0".to_string());
        
        let alert2 = ResourceAlert::new(
            AlertType::HighMemoryUsage,
            AlertLevel::Critical,
            "メモリ使用率が95%を超えています".to_string()
        );
        
        manager.add_alert(alert1);
        manager.add_alert(alert2);
        
        // アクティブアラート数の確認
        assert_eq!(manager.active_alerts_count(), 2);
        
        // 重要度でフィルタリング
        let critical_alerts = manager.get_alerts_by_level(AlertLevel::Critical);
        assert_eq!(critical_alerts.len(), 1);
        assert_eq!(critical_alerts[0].alert_type, AlertType::HighMemoryUsage);
        
        // タイプでフィルタリング
        let cpu_alerts = manager.get_alerts_by_type(AlertType::HighCpuUsage);
        assert_eq!(cpu_alerts.len(), 1);
        
        // リソースでフィルタリング
        let cpu0_alerts = manager.get_alerts_by_resource("cpu0");
        assert_eq!(cpu0_alerts.len(), 1);
        
        // 最も重要なアラートの取得
        if let Some(most_critical) = manager.most_critical_alert() {
            assert_eq!(most_critical.level, AlertLevel::Critical);
            assert_eq!(most_critical.alert_type, AlertType::HighMemoryUsage);
        } else {
            panic!("most_critical_alert should return Some");
        }
        
        // タイプによる解決
        let resolved = manager.resolve_alerts_by_type(AlertType::HighCpuUsage, Some("cpu0".to_string()));
        assert_eq!(resolved.len(), 1);
        assert_eq!(manager.active_alerts_count(), 1); // 1つ解決したので残り1つ
        assert_eq!(manager.history_alerts_count(), 1); // 履歴に1つ追加
        
        // IDによる解決（残りのアラート）
        let remaining_id = manager.active_alerts[0].id.clone();
        manager.resolve_alert(&remaining_id);
        
        assert_eq!(manager.active_alerts_count(), 0); // すべて解決した
        assert_eq!(manager.history_alerts_count(), 2); // 履歴に2つ
    }
    
    #[test]
    fn test_should_update() {
        // 基本アラート
        let base_alert = ResourceAlert::new(
            AlertType::HighCpuUsage,
            AlertLevel::Warning,
            "CPU使用率が高い".to_string()
        ).with_resource_id("cpu0".to_string())
        .with_value(90.0, Some("%".to_string()));
        
        // 同じタイプだが値が少し変わっただけのアラート（更新すべきでない）
        let similar_alert = ResourceAlert::new(
            AlertType::HighCpuUsage,
            AlertLevel::Warning,
            "CPU使用率が高い".to_string()
        ).with_resource_id("cpu0".to_string())
        .with_value(91.0, Some("%".to_string()));
        
        assert!(!similar_alert.should_update(&base_alert));
        
        // 値が大幅に変化したアラート（更新すべき）
        let changed_value_alert = ResourceAlert::new(
            AlertType::HighCpuUsage,
            AlertLevel::Warning,
            "CPU使用率が高い".to_string()
        ).with_resource_id("cpu0".to_string())
        .with_value(75.0, Some("%".to_string()));
        
        assert!(changed_value_alert.should_update(&base_alert));
        
        // レベルが変化したアラート（更新すべき）
        let changed_level_alert = ResourceAlert::new(
            AlertType::HighCpuUsage,
            AlertLevel::Critical,
            "CPU使用率が非常に高い".to_string()
        ).with_resource_id("cpu0".to_string())
        .with_value(95.0, Some("%".to_string()));
        
        assert!(changed_level_alert.should_update(&base_alert));
    }
    
    #[test]
    fn test_should_notify() {
        // 情報レベル・電源変更アラート（通知すべき）
        let power_alert = ResourceAlert::new(
            AlertType::PowerSourceChanged,
            AlertLevel::Info,
            "ACアダプターに接続されました".to_string()
        );
        
        assert!(power_alert.should_notify());
        
        // 情報レベル・その他のアラート（通知すべきでない）
        let info_alert = ResourceAlert::new(
            AlertType::Other,
            AlertLevel::Info,
            "情報メッセージ".to_string()
        );
        
        assert!(!info_alert.should_notify());
        
        // 警告レベルのアラート（通知すべき）
        let warning_alert = ResourceAlert::new(
            AlertType::HighCpuUsage,
            AlertLevel::Warning,
            "CPU使用率が高い".to_string()
        );
        
        assert!(warning_alert.should_notify());
        
        // 重大レベルのアラート（通知すべき）
        let critical_alert = ResourceAlert::new(
            AlertType::HighTemperature,
            AlertLevel::Critical,
            "温度が非常に高い".to_string()
        );
        
        assert!(critical_alert.should_notify());
    }
} 