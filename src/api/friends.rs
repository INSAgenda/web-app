use js_sys::encode_uri_component;

use super::*;

pub async fn get_friends() -> Result<FriendsLists, ApiError> {
    api_get("friends/").await
}

pub async fn request_friend(email: String) -> Result<(), ApiError> {
    api_post_form(&format!("email={}", encode_uri_component(&email)), "friends/request").await
}
