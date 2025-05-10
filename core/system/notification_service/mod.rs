use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationCategory {
    System,
    Security,
    Network,
    Hardware,
    Application,
    Update,
    Calendar,
    Email,
    Message,
    Social,
    Media,
    Weather,
    Health,
    Custom(String),
}

impl NotificationCategory {
    pub fn as_str(&self) -> &str {
        match self {
            NotificationCategory::System => "system",
            NotificationCategory::Security => "security",
            NotificationCategory::Network => "network",
            NotificationCategory::Hardware => "hardware",
            NotificationCategory::Application => "application",
            NotificationCategory::Update => "update",
            NotificationCategory::Calendar => "calendar",
            NotificationCategory::Email => "email",
            NotificationCategory::Message => "message",
            NotificationCategory::Social => "social",
            NotificationCategory::Media => "media",
            NotificationCategory::Weather => "weather",
            NotificationCategory::Health => "health",
            NotificationCategory::Custom(name) => name.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "system" => NotificationCategory::System,
            "security" => NotificationCategory::Security,
            "network" => NotificationCategory::Network,
            "hardware" => NotificationCategory::Hardware,
            "application" => NotificationCategory::Application,
            "update" => NotificationCategory::Update,
            "calendar" => NotificationCategory::Calendar,
            "email" => NotificationCategory::Email,
            "message" => NotificationCategory::Message,
            "social" => NotificationCategory::Social,
            "media" => NotificationCategory::Media,
            "weather" => NotificationCategory::Weather,
            "health" => NotificationCategory::Health,
            _ => NotificationCategory::Custom(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl NotificationPriority {
    pub fn as_i32(&self) -> i32 {
        match self {
            NotificationPriority::Low => 0,
            NotificationPriority::Normal => 1,
            NotificationPriority::High => 2,
            NotificationPriority::Critical => 3,
        }
    }

    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => NotificationPriority::Low,
            1 => NotificationPriority::Normal,
            2 => NotificationPriority::High,
            3 => NotificationPriority::Critical,
            _ => NotificationPriority::Normal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub callback_data: Option<String>,
}

impl NotificationAction {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            icon: None,
            callback_data: None,
        }
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn with_callback_data(mut self, data: &str) -> Self {
        self.callback_data = Some(data.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub app_id: String,
    pub app_name: String,
    pub app_icon: Option<String>,
    pub category: NotificationCategory,
    pub priority: NotificationPriority,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub sound: Option<String>,
    pub actions: Vec<NotificationAction>,
    pub persistent: bool,
    pub timestamp: SystemTime,
    pub expiration: Option<SystemTime>,
    pub user_interaction: bool,
    pub read: bool,
}

impl Notification {
    pub fn new(app_id: &str, app_name: &str, title: &str, body: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            app_id: app_id.to_string(),
            app_name: app_name.to_string(),
            app_icon: None,
            category: NotificationCategory::Application,
            priority: NotificationPriority::Normal,
            title: title.to_string(),
            body: body.to_string(),
            icon: None,
            sound: None,
            actions: Vec::new(),
            persistent: false,
            timestamp: SystemTime::now(),
            expiration: None,
            user_interaction: false,
            read: false,
        }
    }

    pub fn with_category(mut self, category: NotificationCategory) -> Self {
        self.category = category;
        self
    }

    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn with_app_icon(mut self, icon: &str) -> Self {
        self.app_icon = Some(icon.to_string());
        self
    }

    pub fn with_sound(mut self, sound: &str) -> Self {
        self.sound = Some(sound.to_string());
        self
    }

    pub fn with_action(mut self, action: NotificationAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_expiration(mut self, expiration: SystemTime) -> Self {
        self.expiration = Some(expiration);
        self
    }

    pub fn with_expiration_duration(mut self, duration: Duration) -> Self {
        self.expiration = Some(SystemTime::now() + duration);
        self
    }

    pub fn make_persistent(mut self) -> Self {
        self.persistent = true;
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration {
            return SystemTime::now() > expiration;
        }
        false
    }

    pub fn mark_as_read(&mut self) {
        self.read = true;
    }

    pub fn mark_as_interacted(&mut self) {
        self.user_interaction = true;
    }

    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.timestamp)
            .unwrap_or(Duration::from_secs(0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationDoNotDisturbLevel {
    Off,
    Priority,
    Total,
}

impl NotificationDoNotDisturbLevel {
    pub fn allows_notification(&self, priority: &NotificationPriority) -> bool {
        match self {
            NotificationDoNotDisturbLevel::Off => true,
            NotificationDoNotDisturbLevel::Priority => {
                matches!(priority, NotificationPriority::High | NotificationPriority::Critical)
            }
            NotificationDoNotDisturbLevel::Total => {
                matches!(priority, NotificationPriority::Critical)
            }
        }
    }
}

pub type NotificationId = String;
pub type AppId = String;
pub type NotificationHandler = Arc<dyn Fn(&Notification) + Send + Sync>;
pub type NotificationActionHandler = Arc<dyn Fn(&str, &NotificationAction) + Send + Sync>;

#[derive(Clone)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub do_not_disturb: NotificationDoNotDisturbLevel,
    pub do_not_disturb_scheduled: bool,
    pub do_not_disturb_start_time: (u8, u8), // (hour, minute) in 24h format
    pub do_not_disturb_end_time: (u8, u8),   // (hour, minute) in 24h format
    pub sound_enabled: bool,
    pub vibration_enabled: bool,
    pub max_notifications: usize,
    pub notification_timeout: Duration,      // Default timeout for non-persistent notifications
    pub group_by_app: bool,
    pub allowed_categories: Option<Vec<NotificationCategory>>, // None means all categories allowed
    pub blocked_apps: Vec<AppId>,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            do_not_disturb: NotificationDoNotDisturbLevel::Off,
            do_not_disturb_scheduled: false,
            do_not_disturb_start_time: (22, 0),   // 10:00 PM
            do_not_disturb_end_time: (7, 0),      // 7:00 AM
            sound_enabled: true,
            vibration_enabled: true,
            max_notifications: 100,
            notification_timeout: Duration::from_secs(30),
            group_by_app: true,
            allowed_categories: None,
            blocked_apps: Vec::new(),
        }
    }
}

pub struct NotificationService {
    notifications: Arc<RwLock<HashMap<NotificationId, Notification>>>,
    notification_queue: Arc<Mutex<VecDeque<NotificationId>>>,
    handlers: Arc<RwLock<Vec<NotificationHandler>>>,
    action_handlers: Arc<RwLock<HashMap<String, NotificationActionHandler>>>,
    settings: Arc<RwLock<NotificationSettings>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(RwLock::new(HashMap::new())),
            notification_queue: Arc::new(Mutex::new(VecDeque::new())),
            handlers: Arc::new(RwLock::new(Vec::new())),
            action_handlers: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(NotificationSettings::default())),
        }
    }

    pub fn initialize(&self) -> Result<(), String> {
        // TODO: ここでローカルストレージから設定を読み込む
        // TODO: 過去の通知を読み込む
        Ok(())
    }

    pub fn add_notification(&self, notification: Notification) -> Result<NotificationId, String> {
        // 設定をチェック
        let settings = self.settings.read().unwrap();
        
        if !settings.enabled {
            return Err("通知は無効化されています".to_string());
        }

        // アプリがブロックされていないか確認
        if settings.blocked_apps.contains(&notification.app_id) {
            return Err(format!("アプリ {} は通知をブロックされています", notification.app_id));
        }

        // カテゴリが許可されているか確認
        if let Some(allowed_categories) = &settings.allowed_categories {
            if !allowed_categories.contains(&notification.category) {
                return Err(format!("カテゴリ {:?} は許可されていません", notification.category));
            }
        }

        // DoNotDisturb モードの確認
        let dnd_active = match settings.do_not_disturb {
            NotificationDoNotDisturbLevel::Off => false,
            level => {
                let allows = level.allows_notification(&notification.priority);
                
                if !allows && settings.do_not_disturb_scheduled {
                    // 時間ベースのDNDが有効かチェック
                    let now = chrono::Local::now();
                    let current_time = (now.hour() as u8, now.minute() as u8);
                    
                    let start = settings.do_not_disturb_start_time;
                    let end = settings.do_not_disturb_end_time;
                    
                    // 開始時間が終了時間より後の場合（夜間を跨ぐスケジュール）
                    if start > end {
                        !(current_time > end && current_time < start)
                    } else {
                        current_time >= start && current_time < end
                    }
                } else {
                    !allows
                }
            }
        };

        if dnd_active {
            // 保存はするが、実際の通知は発行しない
            let notification_id = notification.id.clone();
            
            let mut notifications = self.notifications.write().unwrap();
            notifications.insert(notification_id.clone(), notification);
            
            return Ok(notification_id);
        }

        // 通知を保存
        let notification_id = notification.id.clone();
        
        {
            let mut notifications = self.notifications.write().unwrap();
            
            // 最大通知数の確認
            if notifications.len() >= settings.max_notifications {
                // 最も古い通知を削除
                if let Some((oldest_id, _)) = notifications.iter()
                    .filter(|(_, n)| !n.persistent)
                    .min_by_key(|(_, n)| n.timestamp) {
                    let oldest_id = oldest_id.clone();
                    notifications.remove(&oldest_id);
                }
            }
            
            notifications.insert(notification_id.clone(), notification.clone());
        }

        // 通知キューに追加
        {
            let mut queue = self.notification_queue.lock().unwrap();
            queue.push_back(notification_id.clone());
        }

        // 通知ハンドラーを呼び出し
        {
            let handlers = self.handlers.read().unwrap();
            for handler in handlers.iter() {
                handler(&notification);
            }
        }

        Ok(notification_id)
    }

    pub fn remove_notification(&self, notification_id: &str) -> Result<(), String> {
        let mut notifications = self.notifications.write().unwrap();
        
        if notifications.remove(notification_id).is_none() {
            return Err(format!("通知 ID {} が見つかりません", notification_id));
        }
        
        Ok(())
    }
    
    pub fn get_notification(&self, notification_id: &str) -> Option<Notification> {
        let notifications = self.notifications.read().unwrap();
        notifications.get(notification_id).cloned()
    }
    
    pub fn get_all_notifications(&self) -> Vec<Notification> {
        let notifications = self.notifications.read().unwrap();
        notifications.values().cloned().collect()
    }
    
    pub fn get_notifications_by_app(&self, app_id: &str) -> Vec<Notification> {
        let notifications = self.notifications.read().unwrap();
        notifications.values()
            .filter(|n| n.app_id == app_id)
            .cloned()
            .collect()
    }
    
    pub fn get_notifications_by_category(&self, category: &NotificationCategory) -> Vec<Notification> {
        let notifications = self.notifications.read().unwrap();
        notifications.values()
            .filter(|n| n.category == *category)
            .cloned()
            .collect()
    }
    
    pub fn mark_notification_as_read(&self, notification_id: &str) -> Result<(), String> {
        let mut notifications = self.notifications.write().unwrap();
        
        if let Some(notification) = notifications.get_mut(notification_id) {
            notification.mark_as_read();
            Ok(())
        } else {
            Err(format!("通知 ID {} が見つかりません", notification_id))
        }
    }
    
    pub fn mark_all_as_read(&self) {
        let mut notifications = self.notifications.write().unwrap();
        
        for notification in notifications.values_mut() {
            notification.mark_as_read();
        }
    }
    
    pub fn mark_app_notifications_as_read(&self, app_id: &str) {
        let mut notifications = self.notifications.write().unwrap();
        
        for notification in notifications.values_mut() {
            if notification.app_id == app_id {
                notification.mark_as_read();
            }
        }
    }
    
    pub fn process_action(&self, notification_id: &str, action_id: &str) -> Result<(), String> {
        let notifications = self.notifications.read().unwrap();
        let notification = notifications.get(notification_id)
            .ok_or_else(|| format!("通知 ID {} が見つかりません", notification_id))?;
        
        let action = notification.actions.iter()
            .find(|a| a.id == action_id)
            .ok_or_else(|| format!("アクション ID {} が見つかりません", action_id))?;
        
        // アクションハンドラーを呼び出し
        let action_handlers = self.action_handlers.read().unwrap();
        if let Some(handler) = action_handlers.get(&action.id) {
            handler(notification_id, action);
        }
        
        // 通知が対話的に扱われたことをマーク
        drop(notifications);
        let mut notifications = self.notifications.write().unwrap();
        if let Some(notification) = notifications.get_mut(notification_id) {
            notification.mark_as_interacted();
        }
        
        Ok(())
    }
    
    pub fn register_notification_handler<F>(&self, handler: F)
    where
        F: Fn(&Notification) + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().unwrap();
        handlers.push(Arc::new(handler));
    }
    
    pub fn register_action_handler<F>(&self, action_id: &str, handler: F)
    where
        F: Fn(&str, &NotificationAction) + Send + Sync + 'static,
    {
        let mut action_handlers = self.action_handlers.write().unwrap();
        action_handlers.insert(action_id.to_string(), Arc::new(handler));
    }
    
    pub fn clean_expired_notifications(&self) -> usize {
        let mut notifications = self.notifications.write().unwrap();
        let now = SystemTime::now();
        
        let expired_ids: Vec<String> = notifications.iter()
            .filter(|(_, n)| {
                if n.persistent {
                    return false;
                }
                
                if let Some(expiration) = n.expiration {
                    return now > expiration;
                }
                
                // 設定のデフォルトタイムアウトを使用（古すぎる通知は削除）
                let settings = self.settings.read().unwrap();
                let timeout = settings.notification_timeout;
                
                if let Ok(age) = now.duration_since(n.timestamp) {
                    return age > timeout && n.read;
                }
                
                false
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in &expired_ids {
            notifications.remove(id);
        }
        
        expired_ids.len()
    }
    
    pub fn update_settings(&self, settings: NotificationSettings) {
        let mut current_settings = self.settings.write().unwrap();
        *current_settings = settings;
    }
    
    pub fn get_settings(&self) -> NotificationSettings {
        self.settings.read().unwrap().clone()
    }
}

// 実装例：システム通知の作成方法
pub fn create_system_notification(title: &str, body: &str) -> Notification {
    Notification::new("system", "System", title, body)
        .with_category(NotificationCategory::System)
        .with_priority(NotificationPriority::Normal)
        .with_icon("system-notification")
}

// 実装例：エラー通知の作成方法
pub fn create_error_notification(title: &str, body: &str) -> Notification {
    Notification::new("system", "System", title, body)
        .with_category(NotificationCategory::System)
        .with_priority(NotificationPriority::High)
        .with_icon("error-notification")
}

// 実装例：警告通知の作成方法
pub fn create_warning_notification(title: &str, body: &str) -> Notification {
    Notification::new("system", "System", title, body)
        .with_category(NotificationCategory::System)
        .with_priority(NotificationPriority::High)
        .with_icon("warning-notification")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new("test-app", "Test App", "Test Title", "Test Body");
        assert_eq!(notification.app_id, "test-app");
        assert_eq!(notification.app_name, "Test App");
        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.body, "Test Body");
        assert_eq!(notification.category, NotificationCategory::Application);
        assert_eq!(notification.priority, NotificationPriority::Normal);
        assert!(!notification.persistent);
        assert!(!notification.read);
        assert!(!notification.user_interaction);
    }

    #[test]
    fn test_notification_with_methods() {
        let notification = Notification::new("test-app", "Test App", "Test Title", "Test Body")
            .with_category(NotificationCategory::System)
            .with_priority(NotificationPriority::High)
            .with_icon("test-icon")
            .with_app_icon("app-icon")
            .with_sound("test-sound")
            .make_persistent();

        assert_eq!(notification.category, NotificationCategory::System);
        assert_eq!(notification.priority, NotificationPriority::High);
        assert_eq!(notification.icon, Some("test-icon".to_string()));
        assert_eq!(notification.app_icon, Some("app-icon".to_string()));
        assert_eq!(notification.sound, Some("test-sound".to_string()));
        assert!(notification.persistent);
    }

    #[test]
    fn test_notification_expiration() {
        let now = SystemTime::now();
        let future = now + Duration::from_secs(3600);
        let past = now - Duration::from_secs(3600);

        let not_expired = Notification::new("test-app", "Test App", "Test Title", "Test Body")
            .with_expiration(future);
        let expired = Notification::new("test-app", "Test App", "Test Title", "Test Body")
            .with_expiration(past);

        assert!(!not_expired.is_expired());
        assert!(expired.is_expired());
    }

    #[test]
    fn test_notification_service_basic() {
        let service = NotificationService::new();
        let notification = Notification::new("test-app", "Test App", "Test Title", "Test Body");
        
        let id = service.add_notification(notification.clone()).unwrap();
        assert_eq!(service.get_all_notifications().len(), 1);
        
        let retrieved = service.get_notification(&id).unwrap();
        assert_eq!(retrieved.title, "Test Title");
        
        service.mark_notification_as_read(&id).unwrap();
        let retrieved = service.get_notification(&id).unwrap();
        assert!(retrieved.read);
        
        service.remove_notification(&id).unwrap();
        assert_eq!(service.get_all_notifications().len(), 0);
    }

    #[test]
    fn test_notification_handlers() {
        let service = NotificationService::new();
        let received = Arc::new(Mutex::new(false));
        let received_clone = Arc::clone(&received);
        
        service.register_notification_handler(move |notification| {
            assert_eq!(notification.title, "Handler Test");
            *received_clone.lock().unwrap() = true;
        });
        
        let notification = Notification::new("test-app", "Test App", "Handler Test", "Test Body");
        service.add_notification(notification).unwrap();
        
        assert!(*received.lock().unwrap());
    }

    #[test]
    fn test_action_handling() {
        let service = NotificationService::new();
        let action_handled = Arc::new(Mutex::new(false));
        let action_handled_clone = Arc::clone(&action_handled);
        
        service.register_action_handler("test-action", move |notification_id, action| {
            assert!(notification_id.len() > 0);
            assert_eq!(action.id, "test-action");
            assert_eq!(action.label, "Test Action");
            *action_handled_clone.lock().unwrap() = true;
        });
        
        let action = NotificationAction::new("test-action", "Test Action");
        let notification = Notification::new("test-app", "Test App", "Action Test", "Test Body")
            .with_action(action);
        
        let id = service.add_notification(notification).unwrap();
        service.process_action(&id, "test-action").unwrap();
        
        assert!(*action_handled.lock().unwrap());
        
        let notification = service.get_notification(&id).unwrap();
        assert!(notification.user_interaction);
    }

    #[test]
    fn test_dnd_mode() {
        let service = NotificationService::new();
        
        // DNDを設定
        let mut settings = service.get_settings();
        settings.do_not_disturb = NotificationDoNotDisturbLevel::Total;
        service.update_settings(settings);
        
        // 通常優先度の通知
        let normal = Notification::new("test-app", "Test App", "Normal", "Body")
            .with_priority(NotificationPriority::Normal);
        
        // クリティカル優先度の通知
        let critical = Notification::new("test-app", "Test App", "Critical", "Body")
            .with_priority(NotificationPriority::Critical);
        
        service.add_notification(normal.clone()).unwrap();
        service.add_notification(critical.clone()).unwrap();
        
        // ハンドラーは通常の優先度では呼ばれない
        let normal_handled = Arc::new(Mutex::new(false));
        let normal_handled_clone = Arc::clone(&normal_handled);
        
        let critical_handled = Arc::new(Mutex::new(false));
        let critical_handled_clone = Arc::clone(&critical_handled);
        
        service.register_notification_handler(move |notification| {
            if notification.priority == NotificationPriority::Normal {
                *normal_handled_clone.lock().unwrap() = true;
            } else if notification.priority == NotificationPriority::Critical {
                *critical_handled_clone.lock().unwrap() = true;
            }
        });
        
        // 通知が存在しても、DNDにより通常優先度のハンドラーは呼ばれない
        assert!(!*normal_handled.lock().unwrap());
        // クリティカル優先度はDNDでも通過する
        assert!(*critical_handled.lock().unwrap());
    }
    
    #[test]
    fn test_expiration_cleanup() {
        let service = NotificationService::new();
        
        // 短い有効期限の通知を作成
        let short_expiry = Notification::new("test-app", "Test App", "Expiring Soon", "Body")
            .with_expiration_duration(Duration::from_millis(100));
        
        // 長い有効期限の通知を作成
        let long_expiry = Notification::new("test-app", "Test App", "Expiring Later", "Body")
            .with_expiration_duration(Duration::from_secs(3600));
        
        let short_id = service.add_notification(short_expiry).unwrap();
        let long_id = service.add_notification(long_expiry).unwrap();
        
        assert_eq!(service.get_all_notifications().len(), 2);
        
        // 短い通知の有効期限が切れるのを待つ
        thread::sleep(Duration::from_millis(150));
        
        // 期限切れの通知をクリーンアップ
        let cleaned = service.clean_expired_notifications();
        assert_eq!(cleaned, 1);
        
        // 短い通知は削除され、長い通知は残っているはず
        assert_eq!(service.get_all_notifications().len(), 1);
        assert!(service.get_notification(&short_id).is_none());
        assert!(service.get_notification(&long_id).is_some());
    }
} 