use std::{
    fs,
    sync::{Arc, Mutex},
    time,
};

use crate::{
    database::Database,
    error::FetishResult,
    location::Location,
    models::{
        basic_group_wrapper::BasicGroupWrapper, chat_wrapper::ChatWrapper,
        scouted_chat::ScoutedChat, supergroup_wrapper::SupergroupWrapper,
    },
};
use chrono::Utc;
use log::{debug, info, warn};
use regex::Regex;
use tdlib::{
    enums, functions,
    types::{Chat, ChatMemberStatusRestricted, ChatPermissions},
};

const WALKING_SPEED: f64 = 1.5;
const DAYS_COOLDOWN: i64 = 3;
const CHAT_JOIN_COOLDOW_SECONDS: i64 = 30;
const PUNISHED_FILE_PATH: &str = ".puni";

pub async fn run(
    conn: Arc<Mutex<Database>>,
    locations: Vec<Location>,
    client_id: i32,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> FetishResult<()> {
    info!("Scouting for nearby chats on {} locations", locations.len());
    let mut locations = filter_locations(conn.clone(), locations.clone(), DAYS_COOLDOWN)
        .expect("filtering locations");
    info!("{} remaining locations after filtering", locations.len());

    let mut scouted_location = ScoutedLocation::init(conn.clone()).await?;
    tokio::select! {
        Err(e) = scouted_location.run(&mut locations, client_id) => warn!("Error scouting: {e:#?}"),
        _ = shutdown_rx.recv() => info!("Shutting down scout"),
    }
    info!("Scouting finished");
    Ok(())
}

struct ScoutedLocation {
    conn: Arc<Mutex<Database>>,
    location: Location,
    scouted_at: i64,
    chats: Vec<(i64, Option<i64>)>,
}

impl ScoutedLocation {
    async fn init(conn: Arc<Mutex<Database>>) -> FetishResult<Self> {
        let scouted_location = match ScoutedLocation::from_database(conn.clone())? {
            Some(scouted_location) => {
                info!("Last scouted location at {}", scouted_location.location);
                info!(
                    "{} chats joined and {} chats to join",
                    scouted_location
                        .chats
                        .iter()
                        .filter(|(_, joined_at)| joined_at.is_some())
                        .count(),
                    scouted_location
                        .chats
                        .iter()
                        .filter(|(_, joined_at)| joined_at.is_none())
                        .count()
                );
                scouted_location
            }
            None => {
                info!("No scouted location found in database");
                ScoutedLocation {
                    conn,
                    location: Location::new(f64::MAX, f64::MAX),
                    scouted_at: 0,
                    chats: vec![],
                }
            }
        };
        Ok(scouted_location)
    }

    fn from_database(conn: Arc<Mutex<Database>>) -> FetishResult<Option<Self>> {
        let mut scouted_chats = conn.lock().unwrap().load_all::<ScoutedChat>()?;
        scouted_chats.sort_by(|a, b| b.scouted_at.cmp(&a.scouted_at));
        let Some(last_scouted_chat) = scouted_chats.get(0) else {
            return Ok(None);
        };
        let last_scouted_chat = last_scouted_chat.clone();
        scouted_chats
            .retain(|scouted_chat| scouted_chat.scouted_at == last_scouted_chat.scouted_at);
        Ok(Some(ScoutedLocation {
            conn,
            location: last_scouted_chat.location,
            scouted_at: last_scouted_chat.scouted_at,
            chats: scouted_chats
                .into_iter()
                .map(|scouted_chat| (scouted_chat.chat_id, scouted_chat.joined_at))
                .collect(),
        }))
    }

    async fn run(&mut self, locations: &mut Vec<Location>, client_id: i32) -> FetishResult<()> {
        loop {
            match self.try_join_next_chat(client_id).await? {
                Some(_) => {}
                None => {
                    if let Some(location) = locations.get(0) {
                        if self.location.0 != f64::MAX || self.location.1 != f64::MAX {
                            let route_time = self.location.distance_to(&location) / WALKING_SPEED;
                            let elapsed = Utc::now().timestamp() - self.scouted_at;
                            let remaining_time = route_time as i64 - elapsed;
                            debug!("Route time: {remaining_time} seconds remaining");
                            sleep(remaining_time.min(0).max(1146) as u64).await;
                        }
                        self.scout_location(*location, client_id).await?;
                        locations.remove(0);
                    } else {
                        info!("No more locations to scout");
                        break Ok(());
                    }
                }
            }
        }
    }

    async fn scout_location(&mut self, location: Location, client_id: i32) -> FetishResult<()> {
        info!("Scouting location {location}");
        let scouted_at = Utc::now().timestamp();
        let enums::ChatsNearby::ChatsNearby(chats_nearby) =
            functions::search_chats_nearby(location.into(), client_id).await?;
        info!(
            "Found {} chats nearby",
            chats_nearby.supergroups_nearby.len()
        );
        let mut chats = Vec::new();

        for chat in chats_nearby.supergroups_nearby {
            let conn = self.conn.lock().unwrap();

            let Some(nearby_chat) = conn.load::<ChatWrapper>(chat.chat_id)? else {
                warn!("Chat not found in database: {}", chat.chat_id);
                continue;
            };

            let status = match &nearby_chat.r#type {
                enums::ChatType::Supergroup(supergroup) => {
                    let Some(supergroup) =
                        conn.load::<SupergroupWrapper>(supergroup.supergroup_id)?
                    else {
                        warn!(
                            "Supergroup not found in database: {}",
                            supergroup.supergroup_id
                        );
                        continue;
                    };
                    supergroup.status.clone()
                }
                enums::ChatType::BasicGroup(basic_group) => {
                    let Some(basic_group) =
                        conn.load::<BasicGroupWrapper>(basic_group.basic_group_id)?
                    else {
                        warn!(
                            "Basic group not found in database: {}",
                            basic_group.basic_group_id
                        );
                        continue;
                    };
                    basic_group.status.clone()
                }
                _ => {
                    continue;
                }
            };

            if !can_join_chat(&nearby_chat, &status) {
                info!("Chat '{}' is not suitable", nearby_chat.title);
                continue;
            }

            let joined_at = match status {
                enums::ChatMemberStatus::Restricted(ChatMemberStatusRestricted {
                    is_member: false,
                    ..
                })
                | enums::ChatMemberStatus::Left => None,
                _ => {
                    info!("Already a member of chat '{}'", nearby_chat.title);
                    Some(0)
                }
            };

            chats.push((nearby_chat.id, joined_at));
            conn.save(&ScoutedChat {
                chat_id: nearby_chat.id,
                location,
                scouted_at,
                joined_at,
            })?;
        }

        self.location = location;
        self.scouted_at = scouted_at;
        self.chats = chats;
        Ok(())
    }

    async fn try_join_next_chat(&mut self, client_id: i32) -> FetishResult<Option<()>> {
        if let Some(last_chat_joined_at) = self.get_last_joined_at() {
            let elapsed_since_last_join = Utc::now().timestamp() - last_chat_joined_at;
            if elapsed_since_last_join < CHAT_JOIN_COOLDOW_SECONDS {
                info!(
                    "Chat join cooldown: {} seconds remaining",
                    CHAT_JOIN_COOLDOW_SECONDS - elapsed_since_last_join
                );
                sleep((CHAT_JOIN_COOLDOW_SECONDS - elapsed_since_last_join) as u64).await;
            }
        }

        self.join_next_chat(self.conn.clone(), self.location, self.scouted_at, client_id)
            .await
    }

    fn get_last_joined_at(&self) -> Option<i64> {
        self.chats
            .iter()
            .filter_map(|(_, joined_at)| *joined_at)
            .max()
    }

    fn get_next_unjoined_chat(&mut self) -> Option<&mut (i64, Option<i64>)> {
        self.chats
            .iter_mut()
            .filter(|(_, joined_at)| joined_at.is_none())
            .next()
    }

    async fn join_next_chat(
        &mut self,
        conn: Arc<Mutex<Database>>,
        location: Location,
        scouted_at: i64,
        client_id: i32,
    ) -> FetishResult<Option<()>> {
        if let Some((chat_id, joined_at)) = self.get_next_unjoined_chat() {
            join_chat(*chat_id, client_id).await;
            *joined_at = Some(Utc::now().timestamp());
            conn.lock().unwrap().save(&ScoutedChat {
                chat_id: *chat_id,
                location,
                scouted_at,
                joined_at: *joined_at,
            })?;
            sleep(CHAT_JOIN_COOLDOW_SECONDS as u64).await;
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}

// #[cfg(feature = "check_admin")]
// async fn is_there_admin(supergroup_id: i64, client_id: i32) -> FetishResult<bool> {
//     use tdlib::{
//         enums::{ChatMembers, MessageSender, SupergroupMembersFilter, User, UserType},
//         functions,
//         types::MessageSenderUser,
//     };

//     let ChatMembers::ChatMembers(tdlib::types::ChatMembers { members, .. }) =
//         functions::get_supergroup_members(
//             supergroup_id,
//             Some(SupergroupMembersFilter::Administrators),
//             0,
//             200,
//             client_id,
//         )
//         .await?;

//     for member in members {
//         if let MessageSender::User(MessageSenderUser { user_id }) = member.member_id {
//             let User::User(tdlib::types::User { r#type, .. }) =
//                 functions::get_user(user_id, client_id).await?;
//             if let UserType::Bot(_) = r#type {
//                 continue;
//             }
//             return Ok(true);
//         }
//     }

//     Ok(false)
// }

fn filter_locations(
    db: Arc<Mutex<Database>>,
    locations: Vec<Location>,
    days_cooldown: i64,
) -> FetishResult<Vec<Location>> {
    let days_ago_timestamp = Utc::now()
        .checked_sub_signed(chrono::Duration::days(days_cooldown))
        .expect("invalid timestamp")
        .timestamp();
    let scouted_locations = db
        .lock()
        .unwrap()
        .load_all::<ScoutedChat>()?
        .clone()
        .into_iter()
        .filter_map(|scouted_chat| {
            (scouted_chat.scouted_at > days_ago_timestamp).then(|| scouted_chat.location)
        })
        .collect::<std::collections::HashSet<Location>>()
        .into_iter()
        .collect::<Vec<Location>>();

    Ok(locations
        .into_iter()
        .filter(|location| {
            let location_is_already_scouted = scouted_locations
                .iter()
                .any(|scouted_location| scouted_location.distance_to(location) < 860. / 10.0);
            if location_is_already_scouted {
                debug!(
                    "Location {} was already scouted within the last {DAYS_COOLDOWN} days",
                    location.round(6)
                );
            }
            !location_is_already_scouted
        })
        .collect::<Vec<Location>>())
}

fn can_join_chat(chat: &Chat, status: &enums::ChatMemberStatus) -> bool {
    chat.permissions.can_send_basic_messages
        && match status {
            enums::ChatMemberStatus::Restricted(ChatMemberStatusRestricted {
                permissions:
                    ChatPermissions {
                        can_send_basic_messages: false,
                        ..
                    },
                ..
            })
            | enums::ChatMemberStatus::Banned(_) => false,
            _ => true,
        }

    // let Messages::Messages(messages) =
    //     functions::get_chat_history(chat.chat_id, 0, 0, 100, false, client_id).await?;
    // info!(
    //     "Chat '{}': {} messages and {} admins",
    //     nearby_chat.title, messages.total_count, admins.total_count
    // );
}

async fn join_chat(chat_id: i64, client_id: i32) {
    if let Ok(punished_until) =
        fs::read_to_string(PUNISHED_FILE_PATH).and_then(|s| Ok(s.parse::<i64>().unwrap()))
    {
        if punished_until > Utc::now().timestamp() {
            let punished_for = punished_until - Utc::now().timestamp();
            info!("We're still punished for {punished_for} seconds",);
            sleep(punished_for as u64).await;
        }
        let _ = fs::remove_file(PUNISHED_FILE_PATH);
    }

    info!("Joining chat '{chat_id}'");
    while let Err(e) = functions::join_chat(chat_id, client_id).await {
        warn!("Error joining chat '{chat_id}': {e:#?}");
        if let tdlib::types::Error { code: 429, message } = e {
            // The message is "Too Many Requests: retry after {seconds}"
            if let Some(c) = Regex::new(r"after (\d+)").unwrap().captures(&message) {
                let seconds = c.get(1).unwrap().as_str().parse::<u64>().unwrap();
                warn!("Too many requests, retry in {seconds} seconds");
                let _ = fs::write(
                    PUNISHED_FILE_PATH,
                    format!("{}", Utc::now().timestamp() + seconds as i64),
                );
                sleep(seconds).await;
            }
        }
    }
}

async fn sleep(seconds: u64) {
    info!("Waiting for {seconds} seconds");
    tokio::time::sleep(time::Duration::from_secs(seconds)).await;
}
