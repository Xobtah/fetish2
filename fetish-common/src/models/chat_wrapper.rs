use std::ops::Deref;

use rusqlite::OptionalExtension;
use serde::{Serialize, Serializer};
use tdlib::types::Chat;

use crate::error::FetishResult;

use super::AutoRequestable;

#[derive(Debug)]
pub struct ChatWrapper(Chat);

impl Deref for ChatWrapper {
    type Target = Chat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Chat> for ChatWrapper {
    fn from(chat: Chat) -> Self {
        Self(chat)
    }
}

impl Into<Chat> for ChatWrapper {
    fn into(self) -> Chat {
        self.0
    }
}

impl Serialize for ChatWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl AutoRequestable for ChatWrapper {
    fn create_table_request() -> String {
        r#"CREATE TABLE IF NOT EXISTS CHATS (
            chat_id INTEGER PRIMARY KEY,
            chat_type TEXT NOT NULL,
            title TEXT NOT NULL,
            photo TEXT,
            permissions TEXT NOT NULL,
            last_message TEXT,
            positions TEXT NOT NULL,
            message_sender_id TEXT,
            block_list TEXT,
            has_protected_content BOOLEAN NOT NULL,
            is_translatable BOOLEAN NOT NULL,
            is_marked_as_unread BOOLEAN NOT NULL,
            has_scheduled_messages BOOLEAN NOT NULL,
            can_be_deleted_only_for_self BOOLEAN NOT NULL,
            can_be_deleted_for_all_users BOOLEAN NOT NULL,
            can_be_reported BOOLEAN NOT NULL,
            default_disable_notification BOOLEAN NOT NULL,
            unread_count INTEGER NOT NULL,
            last_read_inbox_message_id INTEGER NOT NULL,
            last_read_outbox_message_id INTEGER NOT NULL,
            unread_mention_count INTEGER NOT NULL,
            unread_reaction_count INTEGER NOT NULL,
            notification_settings TEXT NOT NULL,
            available_reactions TEXT NOT NULL,
            message_auto_delete_time INTEGER NOT NULL,
            background TEXT,
            theme_name TEXT NOT NULL,
            action_bar TEXT,
            video_chat TEXT NOT NULL,
            pending_join_requests TEXT,
            reply_markup_message_id INTEGER NOT NULL,
            draft_message TEXT,
            client_data TEXT NOT NULL
        )"#
        .into()
    }

    fn select_by_id(id: i64, conn: &rusqlite::Connection) -> FetishResult<Option<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM CHATS WHERE chat_id = :chat_id"#)?
            .query_row(
                rusqlite::named_params! {
                    r#":chat_id"#: id,
                },
                |row| {
                    Ok(ChatWrapper(Chat {
                        id,
                        r#type: serde_json::from_str(&row.get::<_, String>("chat_type")?).unwrap(),
                        title: row.get("title")?,
                        photo: serde_json::from_str(&row.get::<_, String>("photo")?).unwrap(),
                        permissions: serde_json::from_str(&row.get::<_, String>("permissions")?)
                            .unwrap(),
                        last_message: None,
                        positions: serde_json::from_str(&row.get::<_, String>("positions")?)
                            .unwrap(),
                        message_sender_id: serde_json::from_str(
                            &row.get::<_, String>("message_sender_id")?,
                        )
                        .unwrap(),
                        block_list: serde_json::from_str(&row.get::<_, String>("block_list")?)
                            .unwrap(),
                        has_protected_content: row.get("has_protected_content")?,
                        is_translatable: row.get("is_translatable")?,
                        is_marked_as_unread: row.get("is_marked_as_unread")?,
                        has_scheduled_messages: row.get("has_scheduled_messages")?,
                        can_be_deleted_only_for_self: row.get("can_be_deleted_only_for_self")?,
                        can_be_deleted_for_all_users: row.get("can_be_deleted_for_all_users")?,
                        can_be_reported: row.get("can_be_reported")?,
                        default_disable_notification: row.get("default_disable_notification")?,
                        unread_count: row.get("unread_count")?,
                        last_read_inbox_message_id: row.get("last_read_inbox_message_id")?,
                        last_read_outbox_message_id: row.get("last_read_outbox_message_id")?,
                        unread_mention_count: row.get("unread_mention_count")?,
                        unread_reaction_count: row.get("unread_reaction_count")?,
                        notification_settings: serde_json::from_str(
                            &row.get::<_, String>("notification_settings")?,
                        )
                        .unwrap(),
                        available_reactions: serde_json::from_str(
                            &row.get::<_, String>("available_reactions")?,
                        )
                        .unwrap(),
                        message_auto_delete_time: row.get("message_auto_delete_time")?,
                        background: serde_json::from_str(&row.get::<_, String>("background")?)
                            .unwrap(),
                        theme_name: row.get("theme_name")?,
                        action_bar: serde_json::from_str(&row.get::<_, String>("action_bar")?)
                            .unwrap(),
                        video_chat: serde_json::from_str(&row.get::<_, String>("video_chat")?)
                            .unwrap(),
                        pending_join_requests: serde_json::from_str(
                            &row.get::<_, String>("pending_join_requests")?,
                        )
                        .unwrap(),
                        reply_markup_message_id: row.get("reply_markup_message_id")?,
                        draft_message: serde_json::from_str(
                            &row.get::<_, String>("draft_message")?,
                        )
                        .unwrap(),
                        client_data: row.get("client_data")?,
                    }))
                },
            )
            .optional()?)
    }

    fn insert(&self, conn: &rusqlite::Connection) -> FetishResult<()> {
        if let Some(_) = Self::select_by_id(self.0.id, conn)? {
            return Ok(());
        }
        conn.execute(
            r#"INSERT INTO CHATS (
            chat_id,
            chat_type,
            title,
            photo,
            permissions,
            positions,
            message_sender_id,
            block_list,
            has_protected_content,
            is_translatable,
            is_marked_as_unread,
            has_scheduled_messages,
            can_be_deleted_only_for_self,
            can_be_deleted_for_all_users,
            can_be_reported,
            default_disable_notification,
            unread_count,
            last_read_inbox_message_id,
            last_read_outbox_message_id,
            unread_mention_count,
            unread_reaction_count,
            notification_settings,
            available_reactions,
            message_auto_delete_time,
            background,
            theme_name,
            action_bar,
            video_chat,
            pending_join_requests,
            reply_markup_message_id,
            draft_message,
            client_data
        ) VALUES (
            :chat_id,
            :chat_type,
            :title,
            :photo,
            :permissions,
            :positions,
            :message_sender_id,
            :block_list,
            :has_protected_content,
            :is_translatable,
            :is_marked_as_unread,
            :has_scheduled_messages,
            :can_be_deleted_only_for_self,
            :can_be_deleted_for_all_users,
            :can_be_reported,
            :default_disable_notification,
            :unread_count,
            :last_read_inbox_message_id,
            :last_read_outbox_message_id,
            :unread_mention_count,
            :unread_reaction_count,
            :notification_settings,
            :available_reactions,
            :message_auto_delete_time,
            :background,
            :theme_name,
            :action_bar,
            :video_chat,
            :pending_join_requests,
            :reply_markup_message_id,
            :draft_message,
            :client_data
        )"#
            .into(),
            rusqlite::named_params! {
                r#":chat_id"#: &self.0.id,
                r#":chat_type"#: &serde_json::to_string(&self.0.r#type).unwrap(),
                r#":title"#: &self.0.title,
                r#":photo"#: &serde_json::to_string(&self.0.photo).unwrap(),
                r#":permissions"#: &serde_json::to_string(&self.0.permissions).unwrap(),
                r#":positions"#: &serde_json::to_string(&self.0.positions).unwrap(),
                r#":message_sender_id"#: &serde_json::to_string(&self.0.message_sender_id).unwrap(),
                r#":block_list"#: &serde_json::to_string(&self.0.block_list).unwrap(),
                r#":has_protected_content"#: &self.0.has_protected_content,
                r#":is_translatable"#: &self.0.is_translatable,
                r#":is_marked_as_unread"#: &self.0.is_marked_as_unread,
                r#":has_scheduled_messages"#: &self.0.has_scheduled_messages,
                r#":can_be_deleted_only_for_self"#: &self.0.can_be_deleted_only_for_self,
                r#":can_be_deleted_for_all_users"#: &self.0.can_be_deleted_for_all_users,
                r#":can_be_reported"#: &self.0.can_be_reported,
                r#":default_disable_notification"#: &self.0.default_disable_notification,
                r#":unread_count"#: &self.0.unread_count,
                r#":last_read_inbox_message_id"#: &self.0.last_read_inbox_message_id,
                r#":last_read_outbox_message_id"#: &self.0.last_read_outbox_message_id,
                r#":unread_mention_count"#: &self.0.unread_mention_count,
                r#":unread_reaction_count"#: &self.0.unread_reaction_count,
                r#":notification_settings"#: &serde_json::to_string(&self.0.notification_settings).unwrap(),
                r#":available_reactions"#: &serde_json::to_string(&self.0.available_reactions).unwrap(),
                r#":message_auto_delete_time"#: &self.0.message_auto_delete_time,
                r#":background"#: &serde_json::to_string(&self.0.background).unwrap(),
                r#":theme_name"#: &self.0.theme_name,
                r#":action_bar"#: &serde_json::to_string(&self.0.action_bar).unwrap(),
                r#":video_chat"#: &serde_json::to_string(&self.0.video_chat).unwrap(),
                r#":pending_join_requests"#: &serde_json::to_string(&self.0.pending_join_requests).unwrap(),
                r#":reply_markup_message_id"#: &self.0.reply_markup_message_id,
                r#":draft_message"#: &serde_json::to_string(&self.0.draft_message).unwrap(),
                r#":client_data"#: &self.0.client_data,
            },
        )?;
        Ok(())
    }
}
