use std::ops::Deref;

use rusqlite::OptionalExtension;
use serde::{Serialize, Serializer};
use tdlib::types::User;

use crate::error::FetishResult;

use super::AutoRequestable;

#[derive(Debug)]
pub struct UserWrapper(User);

impl Deref for UserWrapper {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<User> for UserWrapper {
    fn from(user: User) -> Self {
        Self(user)
    }
}

impl Serialize for UserWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl AutoRequestable for UserWrapper {
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

    fn select_by_id(id: i64, conn: &rusqlite::Connection) -> FetishResult<Option<Self>> {
        Ok(conn
            .prepare(r#"SELECT * FROM USERS WHERE user_id = :user_id"#)?
            .query_row(
                rusqlite::named_params! {
                    r#":user_id"#: id,
                },
                |row| {
                    Ok(UserWrapper(User {
                        id,
                        first_name: row.get("first_name")?,
                        last_name: row.get("last_name")?,
                        usernames: serde_json::from_str(&row.get::<_, String>("usernames")?)
                            .unwrap(),
                        phone_number: row.get("phone_number")?,
                        status: serde_json::from_str(&row.get::<_, String>("status")?).unwrap(),
                        profile_photo: serde_json::from_str(
                            &row.get::<_, String>("profile_photo")?,
                        )
                        .unwrap(),
                        emoji_status: serde_json::from_str(&row.get::<_, String>("emoji_status")?)
                            .unwrap(),
                        is_contact: row.get("is_contact")?,
                        is_mutual_contact: row.get("is_mutual_contact")?,
                        is_close_friend: row.get("is_close_friend")?,
                        is_verified: row.get("is_verified")?,
                        is_premium: row.get("is_premium")?,
                        is_support: row.get("is_support")?,
                        restriction_reason: row.get("restriction_reason")?,
                        is_scam: row.get("is_scam")?,
                        is_fake: row.get("is_fake")?,
                        has_active_stories: row.get("has_active_stories")?,
                        has_unread_active_stories: row.get("has_unread_active_stories")?,
                        have_access: row.get("have_access")?,
                        r#type: serde_json::from_str(&row.get::<_, String>("user_type")?).unwrap(),
                        language_code: row.get("language_code")?,
                        added_to_attachment_menu: row.get("added_to_attachment_menu")?,
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
