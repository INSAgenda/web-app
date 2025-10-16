use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UserInfo {
    /// The count of api keys
    pub api_key_count: u64,
    /// Last password modification timestamp.
    /// Can be `None` if the user has no password or if the user has never changed his password since the addition of the tracking feature.
    pub last_password_mod: Option<i64>,
    /// The email associated with its verification state
    pub email: (String, bool),
    /// Which groups the user is in
    pub groups: Groups,
    /// Which groups the user is officially in
    pub official_groups: Groups,
    /// Which groups the user could be in
    pub available_groups: Groups,
    /// Last colors modification timestamp.
    pub last_colors_mod: i64,
    /// ICS Token
    pub token: String,
    /// Uid
    pub uid: i64,
}
