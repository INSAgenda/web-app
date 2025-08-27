use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Comment {
    /// Random number identifying the comment.
    pub cid: u64,
    /// Id of the parent comment, if any.
    pub parent: Option<u64>,
    /// Author of the comment.
    pub author: UserDesc,
    /// Content of the comment.
    /// Markdown isn't supported but will be eventually.
    pub content: String,
    /// Timestamp of the comment creation.
    pub creation_ts: i64,
    /// Equal to `creation_ts` if the comment has never been edited.
    pub last_edited_ts: i64,
    /// Number of upvotes 
    pub upvotes: u32,
    /// Number of downvotes.
    pub downvotes: u32,
    /// The vote of the current user.
    /// -1, 0 or 1.
    pub vote: i8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommentRequest {
    /// Event id
    pub eid: String,
    /// Random number identifying the comment.
    pub cid: Option<i64>,
    /// Id of the parent comment, if any.
    pub parent: Option<i64>,
    /// Content of the comment.
    /// Markdown isn't supported but will be eventually.
    pub content: String,
}
