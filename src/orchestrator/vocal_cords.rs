//! Vocal Cords (Messaging Bridge)
//! 
//! Provides the agency with a voice on external platforms like Telegram and Matrix.
//! Enables proactive notifications and mobile interaction.

use teloxide::prelude::*;
use matrix_sdk::{Client as MatrixClient, ruma::{OwnedUserId, OwnedRoomId, events::room::message::RoomMessageEventContent}};
use tracing::{info, warn};
use anyhow::Result;
use tokio::sync::OnceCell;

pub struct VocalCords {
    tg_bot: Option<Bot>,
    tg_chat_id: Option<ChatId>,
    matrix_client: OnceCell<MatrixClient>,
    matrix_room_id: Option<String>,
}

impl VocalCords {
    /// Initialize the bridge using environment variables
    pub fn new() -> Self {
        // Telegram Config
        let tg_token = std::env::var("TELEGRAM_BOT_TOKEN").ok();
        let tg_chat_id_str = std::env::var("TELEGRAM_CHAT_ID").ok();
        
        let tg_bot = tg_token.map(Bot::new);
        let tg_chat_id = tg_chat_id_str.and_then(|id| id.parse::<i64>().ok()).map(ChatId);

        // Matrix Config (Lazy Init)
        let matrix_room_id = std::env::var("MATRIX_ROOM_ID").ok();

        if tg_bot.is_some() && tg_chat_id.is_some() {
            info!("🔊 Vocal Cords: Telegram enabled.");
        }
        
        if matrix_room_id.is_some() {
            info!("🔊 Vocal Cords: Matrix configured (lazy init).");
        }

        Self { 
            tg_bot, 
            tg_chat_id, 
            matrix_client: OnceCell::new(),
            matrix_room_id 
        }
    }

    async fn get_matrix_client(&self) -> Option<&MatrixClient> {
        let homeserver = std::env::var("MATRIX_HOMESERVER").ok()?;
        let user_id_str = std::env::var("MATRIX_USER_ID").ok()?;
        let password = std::env::var("MATRIX_PASSWORD").ok()?;

        self.matrix_client.get_or_try_init(|| async {
            info!("🌐 Initializing Matrix client...");
            let user = <OwnedUserId>::try_from(user_id_str.as_str())
                .map_err(|e| anyhow::anyhow!("Invalid Matrix User ID: {}", e))?;
            
            let client = MatrixClient::builder()
                .homeserver_url(homeserver)
                .build()
                .await?;
            
            client.matrix_auth().login_username(user, &password).send().await?;
            info!("✅ Matrix login successful.");
            Ok::<_, anyhow::Error>(client)
        }).await.ok()
    }

    /// Send a proactive message to all active channels
    pub async fn say(&self, message: &str) -> Result<()> {
        // 1. Send to Telegram
        if let (Some(bot), Some(chat_id)) = (&self.tg_bot, self.tg_chat_id) {
            info!("📣 Sending Telegram notification...");
            if let Err(e) = bot.send_message(chat_id, message).await {
                warn!("Telegram notification failed: {}", e);
            }
        }

        // 2. Send to Matrix
        if let Some(room_id_str) = &self.matrix_room_id {
            if let Some(client) = self.get_matrix_client().await {
                info!("📣 Sending Matrix notification...");
                if let Ok(room_id) = <OwnedRoomId>::try_from(room_id_str.as_str()) {
                    let joined_rooms = client.joined_rooms();
                    if let Some(room) = joined_rooms.iter().find(|r| r.room_id() == &room_id) {
                        let content = RoomMessageEventContent::text_plain(message);
                        if let Err(e) = room.send(content).await {
                            warn!("Matrix notification failed: {}", e);
                        }
                    } else {
                        warn!("Matrix: Joined room {} not found.", room_id_str);
                    }
                } else {
                    warn!("Matrix: Invalid Room ID format: {}", room_id_str);
                }
            }
        }

        Ok(())
    }

    /// Whether any vocal channel is active
    pub fn is_active(&self) -> bool {
        let tg_active = self.tg_bot.is_some() && self.tg_chat_id.is_some();
        let matrix_active = self.matrix_room_id.is_some();
        tg_active || matrix_active
    }
}