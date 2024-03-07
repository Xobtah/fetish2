use std::{
    sync::{Arc, Mutex},
    time,
};

use chrono::Utc;
use crate::{
    database::Database,
    error::FetishResult,
    location::Location,
    models::{
        basic_group_wrapper::BasicGroupWrapper, chat_wrapper::ChatWrapper,
        scouted_chat::ScoutedChat, supergroup_wrapper::SupergroupWrapper,
    },
};
use log::{debug, info, warn};
use regex::Regex;
use tdlib::{
    enums, functions,
    types::{Chat, ChatMemberStatusRestricted},
};

const WALKING_SPEED: f64 = 1.5;
const DAYS_COOLDOWN: i64 = 3;
const CHAT_JOIN_COOLDOW_SECONDS: i64 = 30;

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

    let next_location = if let Some(location) = locations.get(0) {
        *location
    } else {
        info!("No locations to scout");
        return Ok(());
    };

    let (mut scouted_location, mut time_to_wait) =
        ScoutedLocation::init(conn.clone(), next_location, client_id).await?;
    if time_to_wait.as_secs() > 0 {
        info!("Route time: {} seconds remaining", time_to_wait.as_secs());
    }

    loop {
        tokio::select! {
            _ = tokio::time::sleep(time_to_wait) => {
                match scouted_location.step(&mut locations, client_id).await {
                    Ok(State::Wait(ttw)) => {
                        info!("Waiting for {} seconds", ttw.as_secs());
                        time_to_wait = ttw;
                    },
                    Ok(State::Done) => break,
                    Err(e) => {
                        warn!("Error scouting: {e:#?}");
                        break;
                    }
                }
            },
            _ = shutdown_rx.recv() => {
                debug!("Shutting down scout");
                break;
            }
        }
    }
    info!("Scouting finished");
    Ok(())
}

enum State {
    Wait(time::Duration),
    Done,
}

struct ScoutedLocation {
    conn: Arc<Mutex<Database>>,
    location: Location,
    scouted_at: i64,
    chats: Vec<(i64, Option<i64>)>,
}

impl ScoutedLocation {
    async fn init(
        conn: Arc<Mutex<Database>>,
        location: Location,
        client_id: i32,
    ) -> FetishResult<(Self, time::Duration)> {
        let scouted_location =
            if let Some(scouted_location) = ScoutedLocation::from_database(conn.clone())? {
                scouted_location
            } else {
                ScoutedLocation::from_location(conn.clone(), location, client_id).await?
            };
        let remaining_time = scouted_location.route_remaining_time(location);
        Ok((scouted_location, remaining_time))
    }

    fn from_database(conn: Arc<Mutex<Database>>) -> FetishResult<Option<Self>> {
        let last_scouted_location = conn
            .lock()
            .unwrap()
            .load_all::<ScoutedChat>()?
            .into_iter()
            .fold(
                std::collections::HashMap::<Location, ScoutedLocation>::new(),
                |mut acc, scouted_chat| {
                    acc.entry(scouted_chat.location)
                        .or_insert(ScoutedLocation {
                            conn: conn.clone(),
                            location: scouted_chat.location,
                            scouted_at: scouted_chat.scouted_at,
                            chats: vec![(scouted_chat.chat_id, scouted_chat.joined_at)],
                        })
                        .chats
                        .push((scouted_chat.chat_id, scouted_chat.joined_at));
                    acc
                },
            )
            .into_values()
            .fold(None, |acc: Option<ScoutedLocation>, scouted_location| {
                if let Some(last_scouted) = acc {
                    if scouted_location.scouted_at > last_scouted.scouted_at {
                        Some(scouted_location)
                    } else {
                        Some(last_scouted)
                    }
                } else {
                    Some(scouted_location)
                }
            });
        Ok(last_scouted_location)
    }

    async fn from_location(
        conn: Arc<Mutex<Database>>,
        location: Location,
        client_id: i32,
    ) -> FetishResult<Self> {
        info!("Scouting location {:?}", location);
        let scouted_at = Utc::now().timestamp();
        let enums::ChatsNearby::ChatsNearby(chats_nearby) =
            functions::search_chats_nearby(location.into(), client_id).await?;
        info!(
            "Found {} chats nearby",
            chats_nearby.supergroups_nearby.len()
        );
        let mut chats = Vec::new();

        for chat in chats_nearby.supergroups_nearby {
            let conn = conn.lock().unwrap();

            let Some(nearby_chat) = conn.load::<ChatWrapper>(chat.chat_id)? else {
                warn!("Chat not found in database: {}", chat.chat_id);
                continue;
            };

            if !can_join_chat(&nearby_chat) {
                info!("Chat '{}' is not suitable", nearby_chat.title);
                continue;
            }

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

        Ok(Self {
            conn,
            location,
            scouted_at,
            chats,
        })
    }

    async fn step(&mut self, locations: &mut Vec<Location>, client_id: i32) -> FetishResult<State> {
        match self.try_join_next_chat(client_id).await? {
            State::Wait(time_to_wait) => Ok(State::Wait(time_to_wait)),
            State::Done => {
                if let Some(location) = locations.get(0) {
                    match self.scout(*location, client_id).await? {
                        State::Wait(time_to_wait) => {
                            locations.remove(0);
                            Ok(State::Wait(time_to_wait))
                        }
                        State::Done => Ok(State::Done),
                    }
                } else {
                    Ok(State::Done)
                }
            }
        }
    }

    fn route_remaining_time(&self, next_location: Location) -> time::Duration {
        if self.chats.iter().all(|(_, joined_at)| joined_at.is_some()) {
            time::Duration::from_secs(0)
        } else {
            let route_time = (self.location.distance_to(&next_location) / WALKING_SPEED) as i64;
            let elapsed_time = Utc::now().timestamp() - self.scouted_at;
            let remaining_time = route_time - elapsed_time;
            if remaining_time < 0 || remaining_time > 30 * 60 {
                time::Duration::from_secs(0)
            } else {
                time::Duration::from_secs((route_time - elapsed_time) as u64)
            }
        }
    }

    async fn scout(&mut self, location: Location, client_id: i32) -> FetishResult<State> {
        let this = Self::from_location(self.conn.clone(), location, client_id).await?;
        let route_time = self.location.distance_to(&location) / WALKING_SPEED;
        self.location = location;
        self.scouted_at = this.scouted_at;
        self.chats = this.chats;
        Ok(State::Wait(time::Duration::from_secs(route_time as u64)))
    }

    async fn try_join_next_chat(&mut self, client_id: i32) -> FetishResult<State> {
        if let Some(last_chat_joined_at) = self.get_last_joined_at() {
            let elapsed_since_last_join = Utc::now().timestamp() - last_chat_joined_at;
            if elapsed_since_last_join < CHAT_JOIN_COOLDOW_SECONDS {
                info!(
                    "Chat join cooldown: {} seconds remaining",
                    CHAT_JOIN_COOLDOW_SECONDS - elapsed_since_last_join
                );
                return Ok(State::Wait(time::Duration::from_secs(
                    (CHAT_JOIN_COOLDOW_SECONDS - elapsed_since_last_join) as u64,
                )));
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
    ) -> FetishResult<State> {
        if let Some((chat_id, joined_at)) = self.get_next_unjoined_chat() {
            if let State::Wait(dur) = join_chat(*chat_id, client_id).await? {
                Ok(State::Wait(dur))
            } else {
                *joined_at = Some(Utc::now().timestamp());
                conn.lock().unwrap().save(&ScoutedChat {
                    chat_id: *chat_id,
                    location,
                    scouted_at,
                    joined_at: *joined_at,
                })?;
                Ok(State::Wait(time::Duration::from_secs(
                    CHAT_JOIN_COOLDOW_SECONDS as u64,
                )))
            }
        } else {
            Ok(State::Done)
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
                    "Location {:?} was already scouted within the last {DAYS_COOLDOWN} days",
                    location.round(6)
                );
            }
            !location_is_already_scouted
        })
        .collect::<Vec<Location>>())
}

fn can_join_chat(chat: &Chat) -> bool {
    chat.permissions.can_send_basic_messages

    // let Messages::Messages(messages) =
    //     functions::get_chat_history(chat.chat_id, 0, 0, 100, false, client_id).await?;
    // info!(
    //     "Chat '{}': {} messages and {} admins",
    //     nearby_chat.title, messages.total_count, admins.total_count
    // );
}

async fn join_chat(chat_id: i64, client_id: i32) -> FetishResult<State> {
    info!("Joining chat '{chat_id}'");
    if let Err(e) = functions::join_chat(chat_id, client_id).await {
        warn!("Error joining chat '{chat_id}': {e:#?}");
        if let tdlib::types::Error { code: 429, message } = e {
            // The message is "Too Many Requests: retry after {seconds}"
            if let Some(c) = Regex::new(r"after (\d+)").unwrap().captures(&message) {
                let seconds = c.get(1).unwrap().as_str().parse::<u64>().unwrap();
                warn!("Too many requests, retry in {seconds} seconds");
                return Ok(State::Wait(time::Duration::from_secs(seconds)));
            }
        }
    }
    Ok(State::Done)
}
