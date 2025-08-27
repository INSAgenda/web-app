use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserDesc {
    pub uid: i64,
    pub email: String,
    pub picture: Option<String>,
}

impl UserDesc {
    /// Creates a new UserDesc. (profile_picture is set to None)
    pub fn new(uid: i64, email: String) -> Self {
        UserDesc {
            uid,
            email,
            picture: None,
        }
    }

    /// Returns the username of the user based on the email address.
    /// 
    /// Example: "edouard.foobar@insa-rouen.fr" -> "edouard.foobar"
    pub fn get_username(&self) -> String {
        self.email.split('@').next().unwrap().to_string()
    }

    pub fn get_mastodon_username(&self) -> String {
        self.get_username().replace('.', "_")
    }

    pub fn as_username(&self) -> &str {
        self.email.split('@').next().unwrap()
    }

    pub fn is_admin(&self) -> bool {
        const ADMINS: &[&str] = &["dimitri.timoz", "simon.girard"];
        ADMINS.contains(&self.as_username())
    }

    pub fn is_contributor(&self) -> bool {
        const CONTRIBUTORS: &[&str] = &["dimitri.timoz", "simon.girard", "alix.anneraud", "juline.emond"];
        CONTRIBUTORS.contains(&self.as_username())
    }
}
