use serde::{Serialize, Serializer};
use tdlib::types::User;

use super::AutoRequestable;

pub struct UserWrapper<'a>(&'a User);

impl<'a> From<&'a User> for UserWrapper<'a> {
    fn from(user: &'a User) -> Self {
        Self(user)
    }
}

impl<'a> Serialize for UserWrapper<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'a> AutoRequestable for UserWrapper<'a> {
    fn create_table_request() -> String {
        r#"CREATE TABLE IF NOT EXISTS USERS (
            user_id INTEGER PRIMARY KEY,
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            usernames TEXT,
            phone_number TEXT NOT NULL,
            status TEXT NOT NULL,
            profile_photo TEXT,
            emoji_status TEXT,
            is_contact BOOLEAN NOT NULL,
            is_mutual_contact BOOLEAN NOT NULL,
            is_close_friend BOOLEAN NOT NULL,
            is_verified BOOLEAN NOT NULL,
            is_premium BOOLEAN NOT NULL,
            is_support BOOLEAN NOT NULL,
            restriction_reason TEXT,
            is_scam BOOLEAN NOT NULL,
            is_fake BOOLEAN NOT NULL,
            has_active_stories BOOLEAN NOT NULL,
            has_unread_active_stories BOOLEAN NOT NULL,
            have_access BOOLEAN NOT NULL,
            user_type TEXT NOT NULL,
            language_code TEXT NOT NULL,
            added_to_attachment_menu BOOLEAN NOT NULL
        )"#
        .into()
    }

    fn insert(&self, conn: &rusqlite::Connection) -> crate::error::FetishResult<()> {
        conn.execute(
            r#"INSERT INTO USERS (
            user_id,
            first_name,
            last_name,
            usernames,
            phone_number,
            status,
            profile_photo,
            emoji_status,
            is_contact,
            is_mutual_contact,
            is_close_friend,
            is_verified,
            is_premium,
            is_support,
            restriction_reason,
            is_scam,
            is_fake,
            has_active_stories,
            has_unread_active_stories,
            have_access,
            user_type,
            language_code,
            added_to_attachment_menu
        ) VALUES (
            :user_id,
            :first_name,
            :last_name,
            :usernames,
            :phone_number,
            :status,
            :profile_photo,
            :emoji_status,
            :is_contact,
            :is_mutual_contact,
            :is_close_friend,
            :is_verified,
            :is_premium,
            :is_support,
            :restriction_reason,
            :is_scam,
            :is_fake,
            :has_active_stories,
            :has_unread_active_stories,
            :have_access,
            :user_type,
            :language_code,
            :added_to_attachment_menu
        )"#
            .into(),
            rusqlite::named_params! {
                r#":user_id"#: &self.0.id,
                r#":first_name"#: &self.0.first_name,
                r#":last_name"#: &self.0.last_name,
                r#":usernames"#: &serde_json::to_string(&self.0.usernames).unwrap(),
                r#":phone_number"#: &self.0.phone_number,
                r#":status"#: &serde_json::to_string(&self.0.status).unwrap(),
                r#":profile_photo"#: &serde_json::to_string(&self.0.profile_photo).unwrap(),
                r#":emoji_status"#: &serde_json::to_string(&self.0.emoji_status).unwrap(),
                r#":is_contact"#: &self.0.is_contact,
                r#":is_mutual_contact"#: &self.0.is_mutual_contact,
                r#":is_close_friend"#: &self.0.is_close_friend,
                r#":is_verified"#: &self.0.is_verified,
                r#":is_premium"#: &self.0.is_premium,
                r#":is_support"#: &self.0.is_support,
                r#":restriction_reason"#: &self.0.restriction_reason,
                r#":is_scam"#: &self.0.is_scam,
                r#":is_fake"#: &self.0.is_fake,
                r#":has_active_stories"#: &self.0.has_active_stories,
                r#":has_unread_active_stories"#: &self.0.has_unread_active_stories,
                r#":have_access"#: &self.0.have_access,
                r#":user_type"#: &serde_json::to_string(&self.0.r#type).unwrap(),
                r#":language_code"#: &self.0.language_code,
                r#":added_to_attachment_menu"#: &self.0.added_to_attachment_menu,
            },
        )?;
        Ok(())
    }
}
