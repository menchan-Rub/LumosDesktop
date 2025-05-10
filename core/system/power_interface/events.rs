//! 電源関連イベントの定義
//!
//! このモジュールは、電源状態の変化、バッテリーレベルの変化、システム電源操作など、
//! 電源インターフェースが生成する様々なイベントタイプを定義します。

use std::fmt;
use super::{BatteryHealth, ChargingState, PowerPlan, PowerSource};

/// 電源関連イベント
#[derive(Debug, Clone)]
pub enum PowerEvent {
    /// 電源タイプが変更された（AC→バッテリー、またはその逆）
    PowerSourceChanged {
        /// 以前の電源タイプ
        old: PowerSource,
        /// 新しい電源タイプ
        new: PowerSource,
    },
    
    /// AC電源が接続された
    ACAdapterConnected,
    
    /// AC電源が切断された
    ACAdapterDisconnected,
    
    /// 電源プランが変更された
    PowerPlanChanged {
        /// 以前の電源プラン
        old: PowerPlan,
        /// 新しい電源プラン
        new: PowerPlan,
    },
    
    /// バッテリーが接続された
    BatteryConnected {
        /// バッテリーID
        battery_id: String,
        /// バッテリー名
        name: String,
    },
    
    /// バッテリーが切断された
    BatteryDisconnected {
        /// バッテリーID
        battery_id: String,
    },
    
    /// バッテリー充電レベルが変更された
    BatteryLevelChanged {
        /// バッテリーID
        battery_id: String,
        /// 以前の充電レベル（%）
        old: u8,
        /// 新しい充電レベル（%）
        new: u8,
    },
    
    /// バッテリー充電状態が変更された（充電中、放電中など）
    ChargingStateChanged {
        /// バッテリーID
        battery_id: String,
        /// 以前の充電状態
        old: ChargingState,
        /// 新しい充電状態
        new: ChargingState,
    },
    
    /// バッテリー健康状態が変更された
    BatteryHealthChanged {
        /// バッテリーID
        battery_id: String,
        /// 以前の健康状態
        old: BatteryHealth,
        /// 新しい健康状態
        new: BatteryHealth,
    },
    
    /// バッテリーが満充電になった
    BatteryFullyCharged {
        /// バッテリーID
        battery_id: String,
    },
    
    /// バッテリー残量低下警告
    LowBatteryWarning {
        /// バッテリーID
        battery_id: String,
        /// 現在のバッテリーレベル（%）
        level: u8,
    },
    
    /// バッテリー残量危険警告
    CriticalBatteryWarning {
        /// バッテリーID
        battery_id: String,
        /// 現在のバッテリーレベル（%）
        level: u8,
    },
    
    /// システムがスリープ状態に入る直前
    SystemSleepPending,
    
    /// システムが休止状態に入る直前
    SystemHibernatePending,
    
    /// システムがシャットダウンする直前
    SystemShutdownPending,
    
    /// システムが再起動する直前
    SystemRebootPending,
    
    /// システムがスリープ状態から復帰した
    SystemResumedFromSleep,
    
    /// システムが休止状態から復帰した
    SystemResumedFromHibernate,
    
    /// 電源ボタンが押された
    PowerButtonPressed,
    
    /// スリープボタンが押された
    SleepButtonPressed,
    
    /// ノートPCのふたが閉じられた
    LidClosed,
    
    /// ノートPCのふたが開かれた
    LidOpened,
}

impl PowerEvent {
    /// イベントの説明文を取得
    pub fn description(&self) -> String {
        match self {
            PowerEvent::PowerSourceChanged { old, new } => {
                let old_str = match old {
                    PowerSource::AC => "AC電源",
                    PowerSource::Battery => "バッテリー",
                    PowerSource::UPS => "UPS",
                    PowerSource::Unknown => "不明",
                };
                
                let new_str = match new {
                    PowerSource::AC => "AC電源",
                    PowerSource::Battery => "バッテリー",
                    PowerSource::UPS => "UPS",
                    PowerSource::Unknown => "不明",
                };
                
                format!("電源タイプが{}から{}に変更されました", old_str, new_str)
            },
            
            PowerEvent::ACAdapterConnected => 
                "AC電源が接続されました".to_string(),
                
            PowerEvent::ACAdapterDisconnected => 
                "AC電源が切断されました".to_string(),
                
            PowerEvent::PowerPlanChanged { old, new } => {
                let old_str = match old {
                    PowerPlan::HighPerformance => "高パフォーマンス",
                    PowerPlan::Balanced => "バランス",
                    PowerPlan::PowerSaver => "省電力",
                    PowerPlan::Custom(name) => name,
                };
                
                let new_str = match new {
                    PowerPlan::HighPerformance => "高パフォーマンス",
                    PowerPlan::Balanced => "バランス",
                    PowerPlan::PowerSaver => "省電力",
                    PowerPlan::Custom(name) => name,
                };
                
                format!("電源プランが{}から{}に変更されました", old_str, new_str)
            },
            
            PowerEvent::BatteryConnected { battery_id, name } => 
                format!("バッテリー「{}」（ID: {}）が接続されました", name, battery_id),
                
            PowerEvent::BatteryDisconnected { battery_id } => 
                format!("バッテリー（ID: {}）が切断されました", battery_id),
                
            PowerEvent::BatteryLevelChanged { battery_id, old, new } => 
                format!("バッテリー（ID: {}）の充電レベルが{}%から{}%に変化しました", 
                    battery_id, old, new),
                    
            PowerEvent::ChargingStateChanged { battery_id, old, new } => {
                let old_str = match old {
                    ChargingState::Charging => "充電中",
                    ChargingState::Discharging => "放電中",
                    ChargingState::Full => "満充電",
                    ChargingState::NotPresent => "未接続",
                    ChargingState::Unknown => "不明",
                };
                
                let new_str = match new {
                    ChargingState::Charging => "充電中",
                    ChargingState::Discharging => "放電中",
                    ChargingState::Full => "満充電",
                    ChargingState::NotPresent => "未接続",
                    ChargingState::Unknown => "不明",
                };
                
                format!("バッテリー（ID: {}）の充電状態が{}から{}に変化しました", 
                    battery_id, old_str, new_str)
            },
            
            PowerEvent::BatteryHealthChanged { battery_id, old, new } => {
                let old_str = match old {
                    BatteryHealth::Good => "良好",
                    BatteryHealth::Degrading => "劣化中",
                    BatteryHealth::Poor => "要交換",
                    BatteryHealth::Unknown => "不明",
                };
                
                let new_str = match new {
                    BatteryHealth::Good => "良好",
                    BatteryHealth::Degrading => "劣化中",
                    BatteryHealth::Poor => "要交換",
                    BatteryHealth::Unknown => "不明",
                };
                
                format!("バッテリー（ID: {}）の健康状態が{}から{}に変化しました", 
                    battery_id, old_str, new_str)
            },
            
            PowerEvent::BatteryFullyCharged { battery_id } => 
                format!("バッテリー（ID: {}）が満充電になりました", battery_id),
                
            PowerEvent::LowBatteryWarning { battery_id, level } => 
                format!("警告: バッテリー（ID: {}）の残量が低下しています（{}%）", 
                    battery_id, level),
                    
            PowerEvent::CriticalBatteryWarning { battery_id, level } => 
                format!("危険: バッテリー（ID: {}）の残量が非常に低下しています（{}%）", 
                    battery_id, level),
                    
            PowerEvent::SystemSleepPending => 
                "システムはまもなくスリープ状態に入ります".to_string(),
                
            PowerEvent::SystemHibernatePending => 
                "システムはまもなく休止状態に入ります".to_string(),
                
            PowerEvent::SystemShutdownPending => 
                "システムはまもなくシャットダウンします".to_string(),
                
            PowerEvent::SystemRebootPending => 
                "システムはまもなく再起動します".to_string(),
                
            PowerEvent::SystemResumedFromSleep => 
                "システムがスリープ状態から復帰しました".to_string(),
                
            PowerEvent::SystemResumedFromHibernate => 
                "システムが休止状態から復帰しました".to_string(),
                
            PowerEvent::PowerButtonPressed => 
                "電源ボタンが押されました".to_string(),
                
            PowerEvent::SleepButtonPressed => 
                "スリープボタンが押されました".to_string(),
                
            PowerEvent::LidClosed => 
                "ノートPCのふたが閉じられました".to_string(),
                
            PowerEvent::LidOpened => 
                "ノートPCのふたが開かれました".to_string(),
        }
    }
}

impl fmt::Display for PowerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_power_event_description() {
        let event = PowerEvent::PowerSourceChanged {
            old: PowerSource::Battery,
            new: PowerSource::AC,
        };
        assert_eq!(event.description(), "電源タイプがバッテリーからAC電源に変更されました");
        
        let event = PowerEvent::BatteryLevelChanged {
            battery_id: "BAT0".to_string(),
            old: 50,
            new: 45,
        };
        assert_eq!(event.description(), "バッテリー（ID: BAT0）の充電レベルが50%から45%に変化しました");
        
        let event = PowerEvent::LowBatteryWarning {
            battery_id: "BAT0".to_string(),
            level: 15,
        };
        assert_eq!(event.description(), "警告: バッテリー（ID: BAT0）の残量が低下しています（15%）");
    }
    
    #[test]
    fn test_power_event_display() {
        let event = PowerEvent::SystemSleepPending;
        assert_eq!(format!("{}", event), "システムはまもなくスリープ状態に入ります");
        
        let event = PowerEvent::ACAdapterConnected;
        assert_eq!(format!("{}", event), "AC電源が接続されました");
    }
} 