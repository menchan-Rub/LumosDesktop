// LumosDesktop 通知サービス
// ユーザー向け通知の統合的な管理システム

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};
use serde::{Serialize, Deserialize};

use uuid::Uuid;

/// 通知の優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NotificationPriority {
    /// 低優先度
    Low,
    /// 通常優先度
    Normal,
    /// 高優先度
    High,
    /// 緊急
    Critical,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 通知のカテゴリ
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationCategory {
    /// システム通知
    System,
    /// アプリケーション通知
    Application(String),
    /// ユーザー通知
    User,
    /// ネットワーク通知
    Network,
    /// セキュリティ通知
    Security,
    /// 更新通知
    Updates,
    /// カスタムカテゴリ
    Custom(String),
}

impl Default for NotificationCategory {
    fn default() -> Self {
        Self::System
        }
    }

/// 通知のアクション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    /// アクションID
    pub id: String,
    /// アクション表示名
    pub label: String,
    /// アイコンパス（オプション）
    pub icon: Option<String>,
    /// デフォルトアクション
    pub is_default: bool,
}

/// 通知ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotificationId(Uuid);

impl NotificationId {
    /// 新しい通知IDを生成
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// 文字列表現を取得
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
    }

impl Default for NotificationId {
    fn default() -> Self {
        Self::new()
    }
}

/// 通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// 通知ID
    #[serde(skip)]
    pub id: NotificationId,
    /// 通知タイトル
    pub title: String,
    /// 通知本文
    pub body: String,
    /// 通知アイコン
    pub icon: Option<String>,
    /// 通知カテゴリ
    pub category: NotificationCategory,
    /// 通知優先度
    pub priority: NotificationPriority,
    /// 通知アクション
    pub actions: Vec<NotificationAction>,
    /// 通知の作成時間
    pub creation_time: SystemTime,
    /// 通知の有効期限
    pub expiration: Option<SystemTime>,
    /// 通知の送信元アプリケーション
    pub app_id: Option<String>,
    /// 通知が既読かどうか
    pub is_read: bool,
    /// 通知が閉じられたかどうか
    pub is_dismissed: bool,
    /// 追加メタデータ
    pub metadata: HashMap<String, String>,
}

impl Notification {
    /// 新しい通知を作成
    pub fn new<S: Into<String>>(title: S, body: S) -> Self {
        Self {
            id: NotificationId::new(),
            title: title.into(),
            body: body.into(),
            icon: None,
            category: NotificationCategory::default(),
            priority: NotificationPriority::default(),
            actions: Vec::new(),
            creation_time: SystemTime::now(),
            expiration: None,
            app_id: None,
            is_read: false,
            is_dismissed: false,
            metadata: HashMap::new(),
        }
    }
    
    /// 通知にアクションを追加
    pub fn add_action<S: Into<String>>(mut self, id: S, label: S) -> Self {
        self.actions.push(NotificationAction {
            id: id.into(),
            label: label.into(),
            icon: None,
            is_default: false,
        });
        self
    }
    
    /// 通知にデフォルトアクションを追加
    pub fn add_default_action<S: Into<String>>(mut self, id: S, label: S) -> Self {
        self.actions.push(NotificationAction {
            id: id.into(),
            label: label.into(),
            icon: None,
            is_default: true,
        });
        self
    }

    /// 通知カテゴリを設定
    pub fn with_category(mut self, category: NotificationCategory) -> Self {
        self.category = category;
        self
    }

    /// 通知優先度を設定
    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }

    /// 通知アイコンを設定
    pub fn with_icon<S: Into<String>>(mut self, icon: S) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 通知アプリケーションIDを設定
    pub fn from_app<S: Into<String>>(mut self, app_id: S) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    /// 通知有効期限を設定
    pub fn expires_in(mut self, duration: Duration) -> Self {
        self.expiration = Some(SystemTime::now() + duration);
        self
    }

    /// 通知メタデータを追加
    pub fn with_metadata<S: Into<String>>(mut self, key: S, value: S) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 通知を既読にする
    pub fn mark_as_read(&mut self) {
        self.is_read = true;
    }

    /// 通知を閉じる
    pub fn dismiss(&mut self) {
        self.is_dismissed = true;
    }

    /// 通知が期限切れかどうかを確認
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration {
            match SystemTime::now().duration_since(expiration) {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            false
            }
        }
    }

/// 通知リスナー型
pub type NotificationListener = Box<dyn Fn(&Notification) -> bool + Send + Sync + 'static>;

/// 通知アクションハンドラー型
pub type NotificationActionHandler = Box<dyn Fn(&Notification, &str) -> bool + Send + Sync + 'static>;

/// 通知サービス
pub struct NotificationService {
    /// 通知履歴
    notifications: RwLock<VecDeque<Notification>>,
    /// カテゴリのミュート状態
    muted_categories: RwLock<HashMap<NotificationCategory, bool>>,
    /// 通知リスナー
    listeners: RwLock<Vec<NotificationListener>>,
    /// アクションハンドラー
    action_handlers: RwLock<HashMap<String, NotificationActionHandler>>,
    /// 最大履歴サイズ
    max_history_size: usize,
    /// ドゥノット・ディスターブモード
    do_not_disturb: Mutex<bool>,
}

impl NotificationService {
    /// 新しい通知サービスを作成
    pub fn new() -> Self {
        Self {
            notifications: RwLock::new(VecDeque::new()),
            muted_categories: RwLock::new(HashMap::new()),
            listeners: RwLock::new(Vec::new()),
            action_handlers: RwLock::new(HashMap::new()),
            max_history_size: 100,
            do_not_disturb: Mutex::new(false),
        }
    }

    /// 通知を送信
    pub fn send(&self, notification: Notification) -> NotificationId {
        let id = notification.id.clone();
        
        // 通知をキューに追加
        {
            let mut notifications = self.notifications.write().unwrap();
            notifications.push_front(notification.clone());
            
            // 最大サイズを超えた場合は古い通知を削除
            while notifications.len() > self.max_history_size {
                notifications.pop_back();
            }
        }

        // ドゥノット・ディスターブモードと通知カテゴリのミュート状態をチェック
        let should_notify = {
            let do_not_disturb = self.do_not_disturb.lock().unwrap();
                
            if *do_not_disturb && notification.priority != NotificationPriority::Critical {
                false
                    } else {
                let muted_categories = self.muted_categories.read().unwrap();
                !muted_categories.get(&notification.category).unwrap_or(&false)
            }
        };
        
        // リスナーに通知（必要な場合）
        if should_notify {
            let listeners = self.listeners.read().unwrap();
            for listener in listeners.iter() {
                if !listener(&notification) {
                    break;
                }
            }
        }
        
        id
    }
    
    /// 通知リスナーを追加
    pub fn add_listener<F>(&self, listener: F) where F: Fn(&Notification) -> bool + Send + Sync + 'static {
        let mut listeners = self.listeners.write().unwrap();
        listeners.push(Box::new(listener));
    }
    
    /// アクションハンドラーを登録
    pub fn register_action_handler<F, S: Into<String>>(&self, action_id: S, handler: F)
    where
        F: Fn(&Notification, &str) -> bool + Send + Sync + 'static,
    {
        let mut handlers = self.action_handlers.write().unwrap();
        handlers.insert(action_id.into(), Box::new(handler));
        }

    /// 通知アクションを実行
    pub fn trigger_action(&self, notification_id: &NotificationId, action_id: &str) -> bool {
        // 通知を検索
        let notification = {
            let notifications = self.notifications.read().unwrap();
            
            let notification = notifications.iter().find(|n| n.id == *notification_id);
            
            match notification {
                Some(n) => n.clone(),
                None => return false,
            }
        };
        
        // アクションが存在するか確認
        if !notification.actions.iter().any(|a| a.id == action_id) {
            return false;
        }
        
        // ハンドラーを呼び出し
        let handlers = self.action_handlers.read().unwrap();
        
        if let Some(handler) = handlers.get(action_id) {
            handler(&notification, action_id)
        } else {
            // デフォルトのグローバルハンドラーを使用
            if let Some(default_handler) = handlers.get("*") {
                default_handler(&notification, action_id)
            } else {
                false
            }
        }
    }
    
    /// 通知を既読にする
    pub fn mark_as_read(&self, notification_id: &NotificationId) -> bool {
        let mut notifications = self.notifications.write().unwrap();
        
        if let Some(notification) = notifications.iter_mut().find(|n| n.id == *notification_id) {
            notification.mark_as_read();
            true
        } else {
            false
        }
    }
    
    /// 通知を閉じる
    pub fn dismiss(&self, notification_id: &NotificationId) -> bool {
        let mut notifications = self.notifications.write().unwrap();
        
        if let Some(notification) = notifications.iter_mut().find(|n| n.id == *notification_id) {
            notification.dismiss();
            true
        } else {
            false
        }
    }
    
    /// すべての通知を閉じる
    pub fn dismiss_all(&self) {
        let mut notifications = self.notifications.write().unwrap();
        
        for notification in notifications.iter_mut() {
            notification.dismiss();
        }
    }
    
    /// カテゴリをミュート
    pub fn mute_category(&self, category: NotificationCategory) {
        let mut muted_categories = self.muted_categories.write().unwrap();
        muted_categories.insert(category, true);
    }
    
    /// カテゴリのミュートを解除
    pub fn unmute_category(&self, category: NotificationCategory) {
        let mut muted_categories = self.muted_categories.write().unwrap();
        muted_categories.insert(category, false);
    }
    
    /// ドゥノット・ディスターブモードを有効化
    pub fn enable_do_not_disturb(&self) {
        let mut do_not_disturb = self.do_not_disturb.lock().unwrap();
        *do_not_disturb = true;
    }
    
    /// ドゥノット・ディスターブモードを無効化
    pub fn disable_do_not_disturb(&self) {
        let mut do_not_disturb = self.do_not_disturb.lock().unwrap();
        *do_not_disturb = false;
    }
    
    /// ドゥノット・ディスターブモードの状態を取得
    pub fn is_do_not_disturb(&self) -> bool {
        let do_not_disturb = self.do_not_disturb.lock().unwrap();
        *do_not_disturb
    }
    
    /// すべての通知を取得
    pub fn get_all_notifications(&self) -> Vec<Notification> {
        let notifications = self.notifications.read().unwrap();
        notifications.iter().cloned().collect()
    }
    
    /// 未読の通知を取得
    pub fn get_unread_notifications(&self) -> Vec<Notification> {
        let notifications = self.notifications.read().unwrap();
        notifications.iter()
            .filter(|n| !n.is_read && !n.is_dismissed && !n.is_expired())
            .cloned()
            .collect()
    }
    
    /// 特定カテゴリの通知を取得
    pub fn get_notifications_by_category(&self, category: &NotificationCategory) -> Vec<Notification> {
        let notifications = self.notifications.read().unwrap();
        notifications.iter()
            .filter(|n| n.category == *category && !n.is_dismissed && !n.is_expired())
            .cloned()
            .collect()
    }
    
    /// 最大履歴サイズを設定
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
        
        // 既存の通知を調整
        let mut notifications = self.notifications.write().unwrap();
        while notifications.len() > self.max_history_size {
            notifications.pop_back();
        }
    }
    
    /// 期限切れの通知をクリーンアップ
    pub fn cleanup_expired(&self) -> usize {
        let mut notifications = self.notifications.write().unwrap();
        let before_count = notifications.len();
        
        notifications.retain(|n| !n.is_expired());
        
        let after_count = notifications.len();
        before_count - after_count
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new("テストタイトル", "テスト本文")
            .with_priority(NotificationPriority::High)
            .with_category(NotificationCategory::System)
            .with_icon("system-notification")
            .add_action("ok", "OK")
            .add_default_action("view", "詳細を表示");
            
        assert_eq!(notification.title, "テストタイトル");
        assert_eq!(notification.body, "テスト本文");
        assert_eq!(notification.priority, NotificationPriority::High);
        assert_eq!(notification.category, NotificationCategory::System);
        assert_eq!(notification.icon, Some("system-notification".to_string()));
        assert_eq!(notification.actions.len(), 2);
    }

    #[test]
    fn test_notification_service_basic() {
        let service = NotificationService::new();
        
        let notification = Notification::new("テスト", "これはテスト通知です");
        let id = service.send(notification);
        
        let all = service.get_all_notifications();
        assert_eq!(all.len(), 1);
        
        service.mark_as_read(&id);
        let unread = service.get_unread_notifications();
        assert_eq!(unread.len(), 0);
    }

    #[test]
    fn test_notification_listeners() {
        let service = NotificationService::new();
        
        let was_called = Arc::new(AtomicBool::new(false));
        let was_called_clone = Arc::clone(&was_called);
        
        service.add_listener(move |notification| {
            assert_eq!(notification.title, "リスナーテスト");
            was_called_clone.store(true, Ordering::SeqCst);
            true
        });
        
        let notification = Notification::new("リスナーテスト", "これはリスナーテスト用の通知です");
        service.send(notification);
        
        assert!(was_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_action_handlers() {
        let service = NotificationService::new();
        
        let action_triggered = Arc::new(AtomicBool::new(false));
        let action_triggered_clone = Arc::clone(&action_triggered);
        
        service.register_action_handler("test_action", move |notification, action_id| {
            assert_eq!(notification.title, "アクションテスト");
            assert_eq!(action_id, "test_action");
            action_triggered_clone.store(true, Ordering::SeqCst);
            true
        });
        
        let notification = Notification::new("アクションテスト", "これはアクションテスト用の通知です")
            .add_action("test_action", "テストアクション");
            
        let id = service.send(notification);
        
        assert!(service.trigger_action(&id, "test_action"));
        assert!(action_triggered.load(Ordering::SeqCst));
    }
    
    #[test]
    fn test_do_not_disturb() {
        let service = NotificationService::new();
        
        let was_called = Arc::new(AtomicBool::new(false));
        let was_called_clone = Arc::clone(&was_called);
        
        service.add_listener(move |_| {
            was_called_clone.store(true, Ordering::SeqCst);
            true
        });
        
        service.enable_do_not_disturb();
        assert!(service.is_do_not_disturb());
        
        // 通常優先度の通知は配信されない
        let notification = Notification::new("通常通知", "これは通常優先度の通知です")
            .with_priority(NotificationPriority::Normal);
            
        service.send(notification);
        assert!(!was_called.load(Ordering::SeqCst));
        
        // 緊急優先度の通知は配信される
        let critical_notification = Notification::new("緊急通知", "これは緊急優先度の通知です")
            .with_priority(NotificationPriority::Critical);
            
        service.send(critical_notification);
        assert!(was_called.load(Ordering::SeqCst));
    }
} 