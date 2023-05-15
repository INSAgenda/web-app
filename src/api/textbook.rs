use super::*;

#[derive(Serialize, Deserialize)]
struct VoteQuery {
    pub eid: String,
    pub vote: i8,
    pub cid: u64,
}

pub async fn update_vote(eid: impl Into<String>, vote: i8, cid: u64) -> Result<(), ApiError> {
    api_post(VoteQuery { eid: eid.into(), vote, cid }, "vote").await
}

pub async fn update_comment(eid: impl Into<String>, cid: Option<u64>, parent: Option<u64>, content: String) -> Result<(), ApiError> {
    api_post(CommentRequest {
        eid: eid.into(),
        cid: cid.map(|cid| cid as i64),
        parent: parent.map(|parent| parent as i64),
        content,
    }, "comment").await
}
