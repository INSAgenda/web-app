use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FriendRequestIncoming {
    pub from: (UserDesc, Groups),
    pub at_ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FriendRequestOutgoing{
    pub to: (UserDesc, Groups),
    pub at_ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FriendLists {
    pub friends: Vec<(UserDesc, Groups)>,
    pub outgoing: Vec<FriendRequestOutgoing>,
    pub incoming: Vec<FriendRequestIncoming>,
    pub declined: Vec<(UserDesc, Groups)>,
}
