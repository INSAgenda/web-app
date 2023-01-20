use crate::prelude::*;

pub struct FriendsPage {
    request_error: Option<String>,
}

#[derive(Clone, Properties)]
pub struct FriendsProps {
    pub friends: Rc<Option<FriendsLists>>,
    pub app_link: AppLink,
}

impl PartialEq for FriendsProps {
    fn eq(&self, other: &Self) -> bool { self.friends == other.friends }
}

pub enum FriendsMsg {
    Request,
    RequestSuccess,
    RequestError(String),
}

impl Component for FriendsPage {
    type Message = FriendsMsg;
    type Properties = FriendsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            request_error: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FriendsMsg::Request => {
                let el = window().doc().get_element_by_id("friend-request-input").unwrap();
                let input = el.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                let mut email = input.value();
                if email.is_empty() {
                    return false;
                }
                if !email.contains('@') {
                    email.push_str("@insa-rouen.fr");
                }

                let app_link2 = ctx.props().app_link.clone();
                let link2 = ctx.link().clone();
                spawn_local(async move {
                    match request_friend(email).await {
                        Ok(()) => {
                            input.set_value("");
                            let new_friends = get_friends().await.unwrap(); // TODO unwrap
                            link2.send_message(FriendsMsg::RequestSuccess);
                            app_link2.send_message(AppMsg::UpdateFriends(new_friends));
                        }
                        Err(ApiError::Known(e)) if e.kind == "email_not_verified" => app_link2.send_message(AppMsg::SetPage(Page::EmailVerification{ feature: "friends" })),
                        Err(ApiError::Known(e)) => link2.send_message(FriendsMsg::RequestError(e.to_string())),
                        Err(error) => alert(error.to_string()),
                    }
                });

                true
            }
            FriendsMsg::RequestError(err) => {
                self.request_error = Some(err);
                true
            }
            FriendsMsg::RequestSuccess => {
                self.request_error = None;
                false // no need because it will be rerendered after AppMsg::UpdateFriends
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let friends = match &*ctx.props().friends {
            Some(friends) => friends,
            None => return yew::virtual_dom::VNode::from_html_unchecked(AttrValue::from(include_str!("friends_loading.html"))),
        };

        let has_friends = friends.friends_list.len() > 0;
        let names = friends.friends_list.iter().map(|friend| &friend.0.email).collect::<Vec<_>>();
        let picture_iter = names.iter().map(|name| format!("https://api.dicebear.com/5.x/micah/svg?seed={name}", name = name.replace(" ", "+")));
        let alt_iter = names.iter().map(|name| format!("Avatar of {name}"));
        let name_iter = names.iter();

        let has_incoming = friends.friend_requests_incoming.len() > 0;
        let in_names = friends.friend_requests_incoming.iter().map(|friend| &friend.from.0.email).collect::<Vec<_>>();
        let in_picture_iter = in_names.iter().map(|name| format!("https://api.dicebear.com/5.x/micah/svg?seed={name}", name = name.replace(" ", "+")));
        let in_alt_iter = in_names.iter().map(|name| format!("Avatar of {name}"));
        let in_name_iter = in_names.iter();

        let has_outgoing = friends.friend_requests_outgoing.len() > 0;
        let out_names = friends.friend_requests_outgoing.iter().map(|friend| &friend.to.0.email).collect::<Vec<_>>();
        let out_picture_iter = out_names.iter().map(|name| format!("https://api.dicebear.com/5.x/micah/svg?seed={name}", name = name.replace(" ", "+")));
        let out_alt_iter = out_names.iter().map(|name| format!("Avatar of {name}"));
        let out_name_iter = out_names.iter();

        let onclick_request = ctx.link().callback(|_| FriendsMsg::Request);
        let request_error_opt = self.request_error.as_ref();

        let del_name_iter = names.iter().rev();
        let del_value_iter = names.iter().rev().map(|name| name.replace(" ", "+"));

        template_html!("src/friends/friends.html", ...)
    }
}
