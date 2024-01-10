use crate::prelude::*;

pub struct FriendsPage {
    request_error: Option<String>,
}

#[derive(Clone, Properties)]
pub struct FriendsProps {
    pub friends: Rc<Option<FriendLists>>,
    pub app_link: AppLink,
}

impl PartialEq for FriendsProps {
    fn eq(&self, other: &Self) -> bool { self.friends == other.friends }
}

pub enum FriendsMsg {
    Request,
    RequestSuccess,
    RequestError(String),
    Accept(MouseEvent),
    Decline(MouseEvent),
    Cancel(MouseEvent),
    Agenda(MouseEvent),
    Remove,
    Noop,
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
                if email == "rick" || email == "jules.chiron" {
                    ctx.props().app_link.send_message(AppMsg::SetPage(Page::Rick));
                    return false;
                }
                email = email.to_lowercase();
                if !email.contains('@') {
                    email.push_str("@insa-rouen.fr");
                }

                let app_link2 = ctx.props().app_link.clone();
                let link2 = ctx.link().clone();
                spawn_local(async move {
                    match request_friend(email).await {
                        Ok(()) => {
                            input.set_value("");
                            let new_friends = match get_friends().await {
                                Ok(friends) => friends,
                                Err(error) => {
                                    app_link2.send_message(AppMsg::ApiFailure(error));
                                    return;
                                },
                            };
                            link2.send_message(FriendsMsg::RequestSuccess);
                            app_link2.send_message(AppMsg::UpdateFriends(new_friends));
                        }
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
            FriendsMsg::Decline(event) => {
                let target = event.target().unwrap();
                let node = target.dyn_into::<web_sys::Node>().unwrap();
                let mut parent = node.parent_element().unwrap();
                if parent.tag_name() == "BUTTON" {
                    parent = parent.parent_element().unwrap();
                }                
                let uid: i64 = parent.get_attribute("data-uid").unwrap().parse().unwrap();

                let app_link2 = ctx.props().app_link.clone();
                spawn_local(async move {
                    match decline_friend(uid).await {
                        Ok(()) => {
                            let new_friends = match get_friends().await {
                                Ok(friends) => friends,
                                Err(error) => {
                                    app_link2.send_message(AppMsg::ApiFailure(error));
                                    return;
                                },
                            };
                            app_link2.send_message(AppMsg::UpdateFriends(new_friends));
                        }
                        Err(error) => alert(error.to_string()),
                    }
                });

                false
            },
            FriendsMsg::Accept(event) => {
                let target = event.target().unwrap();
                let node = target.dyn_into::<web_sys::Node>().unwrap();
                let mut parent = node.parent_element().unwrap();
                if parent.tag_name() == "BUTTON" {
                    parent = parent.parent_element().unwrap();
                }
                let uid: i64 = parent.get_attribute("data-uid").unwrap().parse().unwrap();

                let app_link2 = ctx.props().app_link.clone();
                spawn_local(async move {
                    match accept_friend(uid).await {
                        Ok(()) => {
                            let new_friends = match get_friends().await {
                                Ok(friends) => friends,
                                Err(error) => {
                                    app_link2.send_message(AppMsg::ApiFailure(error));
                                    return;
                                },
                            };
                            app_link2.send_message(AppMsg::UpdateFriends(new_friends));
                        }
                        Err(error) => alert(error.to_string()),
                    }
                });

                false
            },
            FriendsMsg::Cancel(event) => {
                let target = event.target().unwrap();
                let el = target.dyn_into::<web_sys::Element>().unwrap();
                let uid: i64 = el.get_attribute("data-uid").unwrap().parse().unwrap();

                let app_link2 = ctx.props().app_link.clone();
                spawn_local(async move {
                    match remove_friend(uid).await {
                        Ok(()) => {
                            let new_friends = match get_friends().await {
                                Ok(friends) => friends,
                                Err(error) => {
                                    app_link2.send_message(AppMsg::ApiFailure(error));
                                    return;
                                },
                            };
                            app_link2.send_message(AppMsg::UpdateFriends(new_friends));
                        }
                        Err(error) => alert(error.to_string()),
                    }
                });

                false
            },
            FriendsMsg::Agenda(event) => {
                let target = event.target().unwrap();
                let el = target.dyn_into::<web_sys::Element>().unwrap();
                let pseudo = el.get_attribute("data-pseudo").unwrap();
                ctx.props().app_link.send_message(AppMsg::SetPage(Page::FriendAgenda { pseudo }));
                false
            },
            FriendsMsg::Remove => {
                let el = window().doc().get_element_by_id("friend-remove-input").unwrap();
                let input = el.dyn_into::<web_sys::HtmlSelectElement>().unwrap();
                let email = format!("{}@insa-rouen.fr", input.value());
                let uid = match ctx.props().friends.as_ref() {
                    Some(friends) => friends.friends.iter().find(|f| f.0.email == email).map(|f| f.0.uid),
                    None => None,
                };
                let uid = match uid {
                    Some(uid) => uid,
                    None => {
                        alert(format!("uid to remove is not found for email {email}"));
                        return false;
                    }
                };

                let app_link2 = ctx.props().app_link.clone();
                spawn_local(async move {
                    match remove_friend(uid).await {
                        Ok(()) => {
                            let new_friends = match get_friends().await {
                                Ok(friends) => friends,
                                Err(error) => {
                                    app_link2.send_message(AppMsg::ApiFailure(error));
                                    return;
                                },
                            };
                            app_link2.send_message(AppMsg::UpdateFriends(new_friends));
                        }
                        Err(error) => alert(error.to_string()),
                    }
                });
                
                false
            }
            FriendsMsg::Noop => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let friends = match &*ctx.props().friends {
            Some(friends) => friends,
            None => return yew::virtual_dom::VNode::from_html_unchecked(AttrValue::from(include_str!("friends_loading.html"))),
        };

        let has_friends = !friends.friends.is_empty();
        let names = friends.friends.iter().map(|friend| friend.0.email.trim_end_matches("@insa-rouen.fr")).collect::<Vec<_>>();
        let picture_iter = friends.friends.iter().map(|friend| friend.0.profile_url());
        let alt_iter = names.iter().map(|name| format!("Avatar of {name}"));
        let name_iter = names.iter();
        let friend_pseudo_iter = friends.friends.iter().map(|friend| friend.0.email.trim_end_matches("@insa-rouen.fr").to_string());

        let has_incoming = !friends.incoming.is_empty();
        let in_names = friends.incoming.iter().map(|req| req.from.0.email.trim_end_matches("@insa-rouen.fr")).collect::<Vec<_>>();
        let in_picture_iter = friends.incoming.iter().map(|req| req.from.0.profile_url());
        let in_alt_iter = in_names.iter().map(|name| format!("Avatar of {name}"));
        let in_name_iter = in_names.iter();
        let in_uid_iter = friends.incoming.iter().map(|req| req.from.0.uid.to_string());

        let has_outgoing = !friends.outgoing.is_empty();
        let out_names = friends.outgoing.iter().map(|friend| friend.to.0.email.trim_end_matches("@insa-rouen.fr")).collect::<Vec<_>>();
        let out_picture_iter = friends.outgoing.iter().map(|req| req.to.0.profile_url());
        let out_alt_iter = out_names.iter().map(|name| format!("Avatar of {name}"));
        let out_name_iter = out_names.iter();
        let out_uid_iter = friends.outgoing.iter().map(|req| req.to.0.uid.to_string());

        let onclick_request = ctx.link().callback(|_| FriendsMsg::Request);
        let onkeyup_request = ctx.link().callback(|e: KeyboardEvent| { if e.key() == "Enter" { FriendsMsg::Request } else { FriendsMsg::Noop } });
        let request_error_opt = self.request_error.as_ref();

        let rem_name_iter = names.iter().rev();
        let rem_value_iter = names.iter().rev().map(|name| name.replace(' ', "+"));
        let onclick_remove = ctx.link().callback(|_| FriendsMsg::Remove);

        template_html!(
            "src/friends/friends.html",
            onclick_rick = {ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Rick))},
            onclick_decline = { ctx.link().callback(|e| FriendsMsg::Decline(e)) },
            onclick_accept = { ctx.link().callback(|e| FriendsMsg::Accept(e)) },
            onclick_cancel = { ctx.link().callback(|e| FriendsMsg::Cancel(e)) },
            onclick_agenda = { ctx.link().callback(|e| FriendsMsg::Agenda(e)) },
            ...
        )
    }
}
