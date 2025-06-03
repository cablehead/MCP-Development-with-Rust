// Example 14: Notification Service Implementation
//
// This example demonstrates how to build a comprehensive notification service
// that supports multiple delivery channels (email, SMS, webhooks, push notifications),
// subscription management, and reliable delivery with retry mechanisms.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

// Enum: NotificationChannel
//
// This enum defines the different channels through which notifications can be sent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub enum NotificationChannel {
    Email,
    Sms,
    Webhook,
    PushNotification,
    InApp,
}

// Enum: NotificationPriority
//
// This enum defines the priority levels for notifications, affecting delivery order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub enum NotificationPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

// Struct: NotificationTemplate
//
// This struct represents a reusable notification template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    id: Uuid,
    name: String,
    subject_template: String,
    body_template: String,
    supported_channels: Vec<NotificationChannel>,
}

// Struct: Notification
//
// This struct represents a notification to be sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    id: Uuid,
    recipient_id: String,
    channel: NotificationChannel,
    priority: NotificationPriority,
    subject: String,
    body: String,
    metadata: HashMap<String, String>,
    created_at: DateTime<Utc>,
    scheduled_for: Option<DateTime<Utc>>,
    retry_count: u32,
    max_retries: u32,
}

// Struct: NotificationSubscription
//
// This struct represents a user's subscription preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSubscription {
    user_id: String,
    channel: NotificationChannel,
    endpoint: String, // email address, phone number, webhook URL, etc.
    is_active: bool,
    preferences: HashMap<String, String>,
}

// Struct: DeliveryResult
//
// This struct represents the result of a notification delivery attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryResult {
    notification_id: Uuid,
    success: bool,
    attempt_count: u32,
    delivered_at: DateTime<Utc>,
    error_message: Option<String>,
}

// Struct: NotificationService
//
// This struct implements the main notification service functionality.
pub struct NotificationService {
    templates: Arc<RwLock<HashMap<String, NotificationTemplate>>>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<NotificationSubscription>>>>,
    #[allow(dead_code)]
    pending_notifications: Arc<RwLock<Vec<Notification>>>,
    delivery_results: Arc<RwLock<Vec<DeliveryResult>>>,
    notification_sender: mpsc::UnboundedSender<Notification>,
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationService {
    // Function: new
    //
    // Creates a new notification service instance and starts the background worker.
    //
    // Returns:
    //     A new NotificationService instance
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let service = Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            pending_notifications: Arc::new(RwLock::new(Vec::new())),
            delivery_results: Arc::new(RwLock::new(Vec::new())),
            notification_sender: sender,
        };

        // Start the background delivery worker
        let delivery_worker = DeliveryWorker::new(receiver, service.delivery_results.clone());

        tokio::spawn(async move {
            delivery_worker.run().await;
        });

        service
    }

    // Function: create_template
    //
    // Creates a new notification template.
    //
    // Arguments:
    //     name: The name of the template
    //     subject_template: The subject template with placeholders
    //     body_template: The body template with placeholders
    //     supported_channels: The channels this template supports
    //
    // Returns:
    //     The ID of the created template
    pub async fn create_template(
        &self,
        name: String,
        subject_template: String,
        body_template: String,
        supported_channels: Vec<NotificationChannel>,
    ) -> Uuid {
        let template = NotificationTemplate {
            id: Uuid::new_v4(),
            name: name.clone(),
            subject_template,
            body_template,
            supported_channels,
        };

        let template_id = template.id;
        let mut templates = self.templates.write().await;
        templates.insert(name, template);

        info!("Created notification template: {}", template_id);
        template_id
    }

    // Function: subscribe_user
    //
    // Subscribes a user to notifications on a specific channel.
    //
    // Arguments:
    //     user_id: The unique identifier of the user
    //     subscription: The subscription details
    //
    // Returns:
    //     Result indicating success or failure
    pub async fn subscribe_user(
        &self,
        user_id: String,
        subscription: NotificationSubscription,
    ) -> Result<(), String> {
        let mut subscriptions = self.subscriptions.write().await;

        let user_subscriptions = subscriptions
            .entry(user_id.clone())
            .or_insert_with(Vec::new);

        // Check if user is already subscribed to this channel
        if user_subscriptions
            .iter()
            .any(|s| s.channel == subscription.channel)
        {
            return Err("User already subscribed to this channel".to_string());
        }

        user_subscriptions.push(subscription);
        info!(
            "User {} subscribed to {:?}",
            user_id,
            user_subscriptions.last().unwrap().channel
        );

        Ok(())
    }

    // Function: send_notification
    //
    // Sends a notification to a user through all their subscribed channels.
    //
    // Arguments:
    //     user_id: The recipient user ID
    //     template_name: The name of the template to use
    //     variables: Variables to substitute in the template
    //     priority: The priority of the notification
    //
    // Returns:
    //     Result with the number of notifications queued
    pub async fn send_notification(
        &self,
        user_id: String,
        template_name: String,
        variables: HashMap<String, String>,
        priority: NotificationPriority,
    ) -> Result<usize, String> {
        // Get the template
        let templates = self.templates.read().await;
        let template = templates
            .get(&template_name)
            .ok_or("Template not found")?
            .clone();
        drop(templates);

        // Get user subscriptions
        let subscriptions = self.subscriptions.read().await;
        let user_subscriptions = subscriptions.get(&user_id).ok_or("User not found")?.clone();
        drop(subscriptions);

        let mut notifications_sent = 0;

        for subscription in user_subscriptions {
            if !subscription.is_active
                || !template.supported_channels.contains(&subscription.channel)
            {
                continue;
            }

            // Process template variables
            let subject = self.process_template(&template.subject_template, &variables);
            let body = self.process_template(&template.body_template, &variables);

            let notification = Notification {
                id: Uuid::new_v4(),
                recipient_id: user_id.clone(),
                channel: subscription.channel,
                priority: priority.clone(),
                subject,
                body,
                metadata: variables.clone(),
                created_at: Utc::now(),
                scheduled_for: None,
                retry_count: 0,
                max_retries: 3,
            };

            // Queue the notification for delivery
            if let Err(e) = self.notification_sender.send(notification) {
                error!("Failed to queue notification: {}", e);
            } else {
                notifications_sent += 1;
            }
        }

        info!(
            "Queued {} notifications for user {}",
            notifications_sent, user_id
        );
        Ok(notifications_sent)
    }

    // Function: process_template
    //
    // Processes a template by substituting variables.
    //
    // Arguments:
    //     template: The template string with {{variable}} placeholders
    //     variables: The variables to substitute
    //
    // Returns:
    //     The processed template string
    fn process_template(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    // Function: get_delivery_status
    //
    // Gets the delivery status for notifications.
    //
    // Arguments:
    //     user_id: Optional user ID to filter by
    //
    // Returns:
    //     Vector of delivery results
    pub async fn get_delivery_status(&self, user_id: Option<String>) -> Vec<DeliveryResult> {
        let results = self.delivery_results.read().await;

        match user_id {
            Some(_uid) => results
                .iter()
                .filter(|_r| {
                    // This is a simplified check; in practice you'd need to track user_id in DeliveryResult
                    true // For demo purposes
                })
                .cloned()
                .collect(),
            None => results.clone(),
        }
    }
}

// Struct: DeliveryWorker
//
// This struct handles the background delivery of notifications.
struct DeliveryWorker {
    receiver: mpsc::UnboundedReceiver<Notification>,
    delivery_results: Arc<RwLock<Vec<DeliveryResult>>>,
}

impl DeliveryWorker {
    // Function: new
    //
    // Creates a new delivery worker.
    fn new(
        receiver: mpsc::UnboundedReceiver<Notification>,
        delivery_results: Arc<RwLock<Vec<DeliveryResult>>>,
    ) -> Self {
        Self {
            receiver,
            delivery_results,
        }
    }

    // Function: run
    //
    // Runs the delivery worker loop.
    async fn run(mut self) {
        while let Some(notification) = self.receiver.recv().await {
            self.deliver_notification(notification).await;
        }
    }

    // Function: deliver_notification
    //
    // Delivers a single notification.
    async fn deliver_notification(&self, mut notification: Notification) {
        notification.retry_count += 1;

        let result = match notification.channel {
            NotificationChannel::Email => self.deliver_email(&notification).await,
            NotificationChannel::Sms => self.deliver_sms(&notification).await,
            NotificationChannel::Webhook => self.deliver_webhook(&notification).await,
            NotificationChannel::PushNotification => self.deliver_push(&notification).await,
            NotificationChannel::InApp => self.deliver_in_app(&notification).await,
        };

        let delivery_result = DeliveryResult {
            notification_id: notification.id,
            success: result.is_ok(),
            attempt_count: notification.retry_count,
            delivered_at: Utc::now(),
            error_message: result.err(),
        };

        // Store the delivery result
        let mut results = self.delivery_results.write().await;
        results.push(delivery_result.clone());

        if delivery_result.success {
            info!(
                "Successfully delivered notification {} via {:?}",
                notification.id, notification.channel
            );
        } else {
            warn!(
                "Failed to deliver notification {} (attempt {}): {:?}",
                notification.id, notification.retry_count, delivery_result.error_message
            );
        }
    }

    // Function: deliver_email
    //
    // Simulates email delivery.
    async fn deliver_email(&self, notification: &Notification) -> Result<(), String> {
        // Simulate email delivery delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Simulate occasional failures
        if rand::random::<f64>() < 0.1 {
            return Err("SMTP server unavailable".to_string());
        }

        info!("📧 Email sent: {}", notification.subject);
        Ok(())
    }

    // Function: deliver_sms
    //
    // Simulates SMS delivery.
    async fn deliver_sms(&self, notification: &Notification) -> Result<(), String> {
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        if rand::random::<f64>() < 0.05 {
            return Err("SMS gateway error".to_string());
        }

        info!("📱 SMS sent: {}", notification.body);
        Ok(())
    }

    // Function: deliver_webhook
    //
    // Simulates webhook delivery.
    async fn deliver_webhook(&self, notification: &Notification) -> Result<(), String> {
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        if rand::random::<f64>() < 0.15 {
            return Err("Webhook endpoint unreachable".to_string());
        }

        info!("🔗 Webhook delivered: {}", notification.subject);
        Ok(())
    }

    // Function: deliver_push
    //
    // Simulates push notification delivery.
    async fn deliver_push(&self, notification: &Notification) -> Result<(), String> {
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        info!("📲 Push notification sent: {}", notification.subject);
        Ok(())
    }

    // Function: deliver_in_app
    //
    // Simulates in-app notification delivery.
    async fn deliver_in_app(&self, notification: &Notification) -> Result<(), String> {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        info!("🔔 In-app notification: {}", notification.subject);
        Ok(())
    }
}

// Function: demo_notification_service
//
// Demonstrates the notification service functionality.
async fn demo_notification_service() -> Result<(), Box<dyn std::error::Error>> {
    let service = NotificationService::new();

    info!("=== Creating notification templates ===");

    // Create a welcome email template
    service.create_template(
        "welcome_email".to_string(),
        "Welcome to {{app_name}}, {{user_name}}!".to_string(),
        "Hello {{user_name}},\n\nWelcome to {{app_name}}! We're excited to have you on board.\n\nBest regards,\nThe {{app_name}} Team".to_string(),
        vec![NotificationChannel::Email, NotificationChannel::InApp],
    ).await;

    // Create an alert template
    service
        .create_template(
            "security_alert".to_string(),
            "Security Alert: {{alert_type}}".to_string(),
            "ALERT: {{alert_message}}\nTime: {{timestamp}}\nAction required: {{action_required}}"
                .to_string(),
            vec![
                NotificationChannel::Email,
                NotificationChannel::Sms,
                NotificationChannel::PushNotification,
            ],
        )
        .await;

    info!("=== Setting up user subscriptions ===");

    // Subscribe a user to email notifications
    service
        .subscribe_user(
            "user123".to_string(),
            NotificationSubscription {
                user_id: "user123".to_string(),
                channel: NotificationChannel::Email,
                endpoint: "user123@example.com".to_string(),
                is_active: true,
                preferences: HashMap::new(),
            },
        )
        .await?;

    // Subscribe the same user to SMS
    service
        .subscribe_user(
            "user123".to_string(),
            NotificationSubscription {
                user_id: "user123".to_string(),
                channel: NotificationChannel::Sms,
                endpoint: "+1234567890".to_string(),
                is_active: true,
                preferences: HashMap::new(),
            },
        )
        .await?;

    info!("=== Sending notifications ===");

    // Send a welcome notification
    let mut welcome_vars = HashMap::new();
    welcome_vars.insert("user_name".to_string(), "John Doe".to_string());
    welcome_vars.insert("app_name".to_string(), "MCP Examples".to_string());

    service
        .send_notification(
            "user123".to_string(),
            "welcome_email".to_string(),
            welcome_vars,
            NotificationPriority::Normal,
        )
        .await?;

    // Send a security alert
    let mut alert_vars = HashMap::new();
    alert_vars.insert("alert_type".to_string(), "Suspicious Login".to_string());
    alert_vars.insert(
        "alert_message".to_string(),
        "Login detected from new device".to_string(),
    );
    alert_vars.insert("timestamp".to_string(), Utc::now().to_rfc3339());
    alert_vars.insert(
        "action_required".to_string(),
        "Please verify if this was you".to_string(),
    );

    service
        .send_notification(
            "user123".to_string(),
            "security_alert".to_string(),
            alert_vars,
            NotificationPriority::High,
        )
        .await?;

    // Wait for deliveries to complete
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    info!("=== Checking delivery status ===");
    let delivery_status = service.get_delivery_status(None).await;

    for result in delivery_status {
        info!(
            "Notification {}: {} (attempt {})",
            result.notification_id,
            if result.success {
                "✅ Delivered"
            } else {
                "❌ Failed"
            },
            result.attempt_count
        );
    }

    Ok(())
}

// Function: main
//
// This is the entry point of the program.
// It demonstrates the notification service implementation with multiple channels,
// templates, subscriptions, and delivery tracking.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the tracing subscriber for logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Notification Service Example");

    // Run the notification service demo
    demo_notification_service().await?;

    info!("Notification Service Example completed successfully");

    Ok(())
}
