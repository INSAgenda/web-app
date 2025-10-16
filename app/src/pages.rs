use crate::prelude::*;

/// The page that is currently displayed.
#[derive(Clone, PartialEq)]
pub enum Page {
    Agenda,
    Event { eid: String },
    Friends,
    FriendAgenda { pseudo: String },
    Stotra,
    Settings,
    Onboarding,
    Rick,
}

impl Page {
    pub fn data_and_title(&self) -> (String, &'static str) {
        match self {
            Page::Settings => (String::from("settings"), "Settings"),
            Page::Agenda => (String::from("agenda"), "Agenda"),
            Page::Friends => (String::from("friends"), "Friends"),
            Page::FriendAgenda { pseudo } => (format!("friend-agenda/{pseudo}"), "Friend agenda"),
            Page::Stotra => (String::from("stotra"), "Stotra"),
            Page::Event { eid } => (format!("event/{eid}"), "Event"),
            Page::Onboarding => (String::from("onboarding"), "Onboarding"),
            Page::Rick => (String::from("r"), "Rick"),
        }
    }

    pub fn from_path(path: &str) -> Page {
        match path.trim_start_matches('/').trim_end_matches('/') {
            "agenda" => Page::Agenda,
            event if event.starts_with("event/") => Page::Event { eid: event[6..].to_string() },
            "friends" => Page::Friends,
            friend_agenda if friend_agenda.starts_with("friend-agenda/") => Page::FriendAgenda { pseudo: friend_agenda[14..].to_string() },
            "stotra" => Page::Stotra,
            "settings" => Page::Settings,
            "onboarding" => Page::Onboarding,
            "r" => Page::Rick,
            pathname => {
                alert(format!("Unknown pathname: {pathname:?}"));
                Page::Agenda
            }
        }
    }
}

/// Redirect the user
pub fn redirect(page: &str) {
    let _ = window().location().set_href(page);
    log!("Redirecting to {page}");
}
