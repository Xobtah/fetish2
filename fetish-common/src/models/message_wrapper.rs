use serde::{Serialize, Serializer};
use tdlib::types::Message;

use super::AutoRequestable;

pub struct MessageWrapper<'a>(&'a Message);

impl<'a> From<&'a Message> for MessageWrapper<'a> {
    fn from(message: &'a Message) -> Self {
        Self(message)
    }
}

impl<'a> Serialize for MessageWrapper<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'a> AutoRequestable for MessageWrapper<'a> {
    fn create_table_request() -> String {
        r#"CREATE TABLE IF NOT EXISTS MESSAGES (
            message_id INTEGER PRIMARY KEY,
            sender_id INTEGER NOT NULL,
            chat_id INTEGER NOT NULL,
            sending_state TEXT,
            scheduling_state TEXT,
            is_outgoing BOOLEAN NOT NULL,
            is_pinned BOOLEAN NOT NULL,
            can_be_edited BOOLEAN NOT NULL,
            can_be_forwarded BOOLEAN NOT NULL,
            can_be_saved BOOLEAN NOT NULL,
            can_be_deleted_only_for_self BOOLEAN NOT NULL,
            can_be_deleted_for_all_users BOOLEAN NOT NULL,
            can_get_added_reactions BOOLEAN NOT NULL,
            can_get_statistics BOOLEAN NOT NULL,
            can_get_message_thread BOOLEAN NOT NULL,
            can_get_viewers BOOLEAN NOT NULL,
            can_get_media_timestamp_links BOOLEAN NOT NULL,
            can_report_reactions BOOLEAN NOT NULL,
            has_timestamped_media BOOLEAN NOT NULL,
            is_channel_post BOOLEAN NOT NULL,
            is_topic_message BOOLEAN NOT NULL,
            contains_unread_mention BOOLEAN NOT NULL,
            date INTEGER NOT NULL,
            edit_date INTEGER NOT NULL,
            forward_info TEXT,
            interaction_info TEXT,
            unread_reactions TEXT NOT NULL,
            reply_to TEXT,
            message_thread_id INTEGER NOT NULL,
            self_destruct_type TEXT,
            self_destruct_in REAL NOT NULL,
            auto_delete_in REAL NOT NULL,
            via_bot_user_id INTEGER NOT NULL,
            author_signature TEXT NOT NULL,
            media_album_id INTEGER NOT NULL,
            restriction_reason TEXT NOT NULL,
            content TEXT NOT NULL,
            reply_markup TEXT
        )"#
        .into()
    }

    fn insert(&self, conn: &rusqlite::Connection) -> crate::error::FetishResult<()> {
        conn.execute(
            r#"INSERT INTO MESSAGES (
            message_id,
            sender_id,
            chat_id,
            sending_state,
            scheduling_state,
            is_outgoing,
            is_pinned,
            can_be_edited,
            can_be_forwarded,
            can_be_saved,
            can_be_deleted_only_for_self,
            can_be_deleted_for_all_users,
            can_get_added_reactions,
            can_get_statistics,
            can_get_message_thread,
            can_get_viewers,
            can_get_media_timestamp_links,
            can_report_reactions,
            has_timestamped_media,
            is_channel_post,
            is_topic_message,
            contains_unread_mention,
            date,
            edit_date,
            forward_info,
            interaction_info,
            unread_reactions,
            reply_to,
            message_thread_id,
            self_destruct_type,
            self_destruct_in,
            auto_delete_in,
            via_bot_user_id,
            author_signature,
            media_album_id,
            restriction_reason,
            content,
            reply_markup
        ) VALUES (
            :message_id,
            :sender_id,
            :chat_id,
            :sending_state,
            :scheduling_state,
            :is_outgoing,
            :is_pinned,
            :can_be_edited,
            :can_be_forwarded,
            :can_be_saved,
            :can_be_deleted_only_for_self,
            :can_be_deleted_for_all_users,
            :can_get_added_reactions,
            :can_get_statistics,
            :can_get_message_thread,
            :can_get_viewers,
            :can_get_media_timestamp_links,
            :can_report_reactions,
            :has_timestamped_media,
            :is_channel_post,
            :is_topic_message,
            :contains_unread_mention,
            :date,
            :edit_date,
            :forward_info,
            :interaction_info,
            :unread_reactions,
            :reply_to,
            :message_thread_id,
            :self_destruct_type,
            :self_destruct_in,
            :auto_delete_in,
            :via_bot_user_id,
            :author_signature,
            :media_album_id,
            :restriction_reason,
            :content,
            :reply_markup
        )"#
            .into(),
            rusqlite::named_params! {
                r#":message_id"#: &self.0.id,
                r#":sender_id"#: &serde_json::to_string(&self.0.sender_id).unwrap(),
                r#":chat_id"#: &self.0.chat_id,
                r#":sending_state"#: &serde_json::to_string(&self.0.sending_state).unwrap(),
                r#":scheduling_state"#: &serde_json::to_string(&self.0.scheduling_state).unwrap(),
                r#":is_outgoing"#: &self.0.is_outgoing,
                r#":is_pinned"#: &self.0.is_pinned,
                r#":can_be_edited"#: &self.0.can_be_edited,
                r#":can_be_forwarded"#: &self.0.can_be_forwarded,
                r#":can_be_saved"#: &self.0.can_be_saved,
                r#":can_be_deleted_only_for_self"#: &self.0.can_be_deleted_only_for_self,
                r#":can_be_deleted_for_all_users"#: &self.0.can_be_deleted_for_all_users,
                r#":can_get_added_reactions"#: &self.0.can_get_added_reactions,
                r#":can_get_statistics"#: &self.0.can_get_statistics,
                r#":can_get_message_thread"#: &self.0.can_get_message_thread,
                r#":can_get_viewers"#: &self.0.can_get_viewers,
                r#":can_get_media_timestamp_links"#: &self.0.can_get_media_timestamp_links,
                r#":can_report_reactions"#: &self.0.can_report_reactions,
                r#":has_timestamped_media"#: &self.0.has_timestamped_media,
                r#":is_channel_post"#: &self.0.is_channel_post,
                r#":is_topic_message"#: &self.0.is_topic_message,
                r#":contains_unread_mention"#: &self.0.contains_unread_mention,
                r#":date"#: &self.0.date,
                r#":edit_date"#: &self.0.edit_date,
                r#":forward_info"#: &serde_json::to_string(&self.0.forward_info).unwrap(),
                r#":interaction_info"#: &serde_json::to_string(&self.0.interaction_info).unwrap(),
                r#":unread_reactions"#: &serde_json::to_string(&self.0.unread_reactions).unwrap(),
                r#":reply_to"#: &serde_json::to_string(&self.0.reply_to).unwrap(),
                r#":message_thread_id"#: &self.0.message_thread_id,
                r#":self_destruct_type"#: &serde_json::to_string(&self.0.self_destruct_type).unwrap(),
                r#":self_destruct_in"#: &self.0.self_destruct_in,
                r#":auto_delete_in"#: &self.0.auto_delete_in,
                r#":via_bot_user_id"#: &self.0.via_bot_user_id,
                r#":author_signature"#: &self.0.author_signature,
                r#":media_album_id"#: &self.0.media_album_id,
                r#":restriction_reason"#: &serde_json::to_string(&self.0.restriction_reason).unwrap(),
                r#":content"#: &serde_json::to_string(&self.0.content).unwrap(),
                r#":reply_markup"#: &serde_json::to_string(&self.0.reply_markup).unwrap(),
            },
        )?;
        Ok(())
    }
}
