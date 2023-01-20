use crate::prelude::*;

pub struct FriendsPage;

#[derive(Clone, PartialEq, Properties)]
pub struct FriendsProps {
    pub friends: Rc<Option<FriendsLists>>
}

pub enum FriendsMsg {
}

impl Component for FriendsPage {
    type Message = FriendsMsg;
    type Properties = FriendsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let friends = match &*ctx.props().friends {
            Some(friends) => friends,
            None => return html! {}
        };

        let names = friends.friends_list.iter().map(|friend| &friend.email).collect::<Vec<_>>();
        let picture_iter = names.iter().map(|name| format!("https://api.dicebear.com/5.x/micah/svg?seed={name}", name = name.replace(" ", "+")));
        let alt_iter = names.iter().map(|name| format!("Avatar of {name}"));
        let name_iter = names.iter();

        let in_names = friends.friend_requests_incoming.iter().map(|friend| &friend.from.email).collect::<Vec<_>>();
        let in_picture_iter = in_names.iter().map(|name| format!("https://api.dicebear.com/5.x/micah/svg?seed={name}", name = name.replace(" ", "+")));
        let in_alt_iter = in_names.iter().map(|name| format!("Avatar of {name}"));
        let in_name_iter = in_names.iter();

        let out_names = friends.friend_requests_outgoing.iter().map(|friend| &friend.to.email).collect::<Vec<_>>();
        let out_picture_iter = out_names.iter().map(|name| format!("https://api.dicebear.com/5.x/micah/svg?seed={name}", name = name.replace(" ", "+")));
        let out_alt_iter = out_names.iter().map(|name| format!("Avatar of {name}"));
        let out_name_iter = out_names.iter();

        let del_name_iter = names.iter().rev();
        let del_value_iter = names.iter().rev().map(|name| name.replace(" ", "+"));

        template_html!("src/friends/friends.html", ...)
    }
}
