use crate::prelude::*;

pub struct Popup {
    comments: Option<Vec<Comment>>,
    friend_counter_folded: bool,
    current_color: String,
    color_changed: bool,
}

pub enum PopupMsg {
    TriggerFriendCounter,
    ColorInput,
    ReloadComments,
    Comment,
    CommentsLoaded(Vec<Comment>),
    AppMsg(AppMsg),
}

#[derive(Properties, Clone)]
pub struct PopupProps {
    pub event: RawEvent,
    pub app_link: AppLink,
    pub user_info: Rc<Option<UserInfo>>,
    pub friends: Rc<Option<FriendLists>>,
    pub colors: Rc<Colors>,
}

impl PartialEq for PopupProps {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event
            && self.user_info == other.user_info
            && self.friends == other.friends
            && self.colors.get(&self.event.summary) == other.colors.get(&self.event.summary)
    }
}

impl Component for Popup {
    type Message = PopupMsg;
    type Properties = PopupProps;

    fn create(ctx: &Context<Self>) -> Self {
        let eid = ctx.props().event.eid.clone();
        let link = ctx.link().clone();
        spawn_local(async move {
            match api_get(format!("comments?eid={eid}")).await {
                Ok(new_comments) => link.send_message(PopupMsg::CommentsLoaded(new_comments)),
                Err(ApiError::Known(e)) if e.kind == "textbook_not_found" => link.send_message(PopupMsg::CommentsLoaded(Vec::new())),
                Err(e) => {
                    alert(e.to_string());
                    link.send_message(PopupMsg::CommentsLoaded(Vec::new()));
                },
            }
        });

        let summary = &ctx.props().event.summary;
        let bg_color = ctx.props().colors.get(summary).map(|c| c.to_string()).unwrap_or_else(|| String::from("#CB6CE6"));

        Self {
            comments: None,
            friend_counter_folded: true,
            current_color: bg_color,
            color_changed: false,
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        if self.color_changed {
            let summary = ctx.props().event.summary.to_owned();
            ctx.props().app_link.send_message(AppMsg::UpdateColor { summary, color: std::mem::take(&mut self.current_color) })    
        }
        ctx.props().app_link.send_message(AppMsg::MarkCommentsAsSeen(ctx.props().event.eid.clone()));
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PopupMsg::CommentsLoaded(mut new_comments) => {
                new_comments.sort_by_key(|c| c.downvotes as isize - c.upvotes as isize);
                self.comments = Some(new_comments);
                true
            }
            PopupMsg::ColorInput => {
                let document = window().doc();
                let el = document.get_element_by_id("popup-color-input").unwrap();
                let color = el.dyn_into::<HtmlInputElement>().unwrap().value();
                self.current_color = color;
                self.color_changed = true;
                true
            },
            PopupMsg::ReloadComments => {
                *self = Component::create(ctx);
                false
            }
            PopupMsg::Comment => {
                let el = window().doc().get_element_by_id("comment-textarea-top").unwrap();
                let textarea = el.dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
                let content = textarea.value();
                textarea.set_value("");

                let eid = ctx.props().event.eid.clone();
                let link = ctx.link().clone();
                spawn_local(async move {
                    if let Err(e) = update_comment(eid, None, None, content).await {
                        alert(e.to_string());
                    }
                    link.send_message(PopupMsg::ReloadComments);
                });
                true
            }
            PopupMsg::TriggerFriendCounter => {
                self.friend_counter_folded = !self.friend_counter_folded;
                true
            }
            PopupMsg::AppMsg(msg) => {
                ctx.props().app_link.send_message(msg);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick_close = ctx.props().app_link.callback(move |_| AppMsg::SetPage(Page::Agenda));

        // Friend counter
        let friends: Vec<_> = ctx.props().friends.deref().as_ref().map(|friends| {
            friends.friends.iter().filter(|friend| {
                let name = friend.0.get_username().split('.').nth(1).map(|n| n.to_uppercase()); // TODO real name
                friend.1.matches_with_name(&ctx.props().event.groups, name.as_deref())
            }).map(|f| &f.0).collect()
        }).unwrap_or_default();
        let names = friends.iter().map(|friend| friend.email.trim_end_matches("@insa-rouen.fr")).collect::<Vec<_>>();
        let names_iter = names.iter();
        let friend_count = friends.len();
        let friend_counter_folded = friend_count != 0 && self.friend_counter_folded;
        let friend_counter_unfolded = friend_count != 0 && !self.friend_counter_folded;
        let only_one_friend = friend_count == 1;
        let z_index_iter = 1..friend_count+1;

        let summary = ctx.props().event.summary.to_owned();
        let bg_color = &self.current_color;
        let name = ctx.props().event.format_name();
        let opt_location = ctx.props().event.format_location();

        let comments_loading = self.comments.is_none();
        let comments = Rc::new(self.comments.clone().unwrap_or_default());
        let eid = Rc::new(ctx.props().event.eid.clone());
        let comment_iter = comments.iter().filter(|c| c.parent.is_none()).map(|c| {
            html! {
                <CommentComp
                    eid={Rc::clone(&eid)}
                    comments={Rc::clone(&comments)}
                    cid={c.cid}
                    user_info={Rc::clone(&ctx.props().user_info)}
                    popup_link={ctx.link().clone()} />
            }
        });

        let user_avatar = format!("https://api.dicebear.com/5.x/identicon/svg?seed={}", ctx.props().user_info.as_ref().as_ref().map(|u| u.uid).unwrap_or(0));
        let user_name = ctx.props().user_info.as_ref().as_ref().map(|u| u.email.0.split('@').next().unwrap().to_string()).unwrap_or(String::from("inconnu"));
        let onclick_comment = ctx.link().callback(|_| PopupMsg::Comment);

        template_html!(
            "src/popup/popup.html",
            teachers = {ctx.props().event.teachers.join(", ")},
            time = {ctx.props().event.format_time()},
            name = {&name},
            bg_color = {bg_color.clone()},
            onclick_close = {onclick_close.clone()},
            onclick_fold = {ctx.link().callback(|_| PopupMsg::TriggerFriendCounter)},
            input_color = {ctx.link().callback(|_| PopupMsg::ColorInput)},
            opt_location = {&opt_location},
            event_color = {event_color.clone()},
            alt_iter = { names.iter().map(|name| format!("Avatar of {}", name)) },
            picture_iter = { friends.iter().map(|friend| friend.profile_url()) },
            ...
        )
    }
}
