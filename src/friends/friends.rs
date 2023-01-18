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
        let names = vec!["John Doe", "Edouart Foobar", "Exponenthis"];
        let picture_iter = names.iter().map(|name| format!("https://api.dicebear.com/5.x/micah/svg?seed={name}", name = name.replace(" ", "+")));
        let alt_iter = names.iter().map(|name| format!("Avatar of {name}"));
        let name_iter = names.iter();

        let in_names = vec!["Satoshi Nakamoto", "Susan Wojcicki"];
        let in_picture_iter = in_names.iter().map(|name| format!("https://api.dicebear.com/5.x/micah/svg?seed={name}", name = name.replace(" ", "+")));
        let in_alt_iter = in_names.iter().map(|name| format!("Avatar of {name}"));
        let in_name_iter = in_names.iter();

        template_html!("src/friends/friends.html", ...)
    }
}
