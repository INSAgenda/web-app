use crate::prelude::*;

pub struct FriendsPage;

pub enum FriendsMsg {
}

impl Component for FriendsPage {
    type Message = FriendsMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        template_html!("src/friends/friends.html", ...)
    }
}
