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

        let out_names = vec!["Tyler Durden", "Walter White", "Gordon Freeman"];
        let out_picture_iter = out_names.iter().map(|name| format!("https://api.dicebear.com/5.x/micah/svg?seed={name}", name = name.replace(" ", "+")));
        let out_alt_iter = out_names.iter().map(|name| format!("Avatar of {name}"));
        let out_name_iter = out_names.iter();

        let del_name_iter = names.iter().rev();
        let del_value_iter = names.iter().rev().map(|name| name.replace(" ", "+"));

        template_html!("src/friends/friends.html", ...)
    }
}
