use crate::{prelude::*, slider::width};

pub struct Popup {
    comments: Option<Vec<Comment>>,
    counter: u64,
}

pub enum PopupMsg {
    SaveColors,
    Comment,
    CommentsLoaded(Vec<Comment>),
}

#[derive(Properties, Clone)]
pub struct PopupProps {
    pub event: RawEvent,
    pub agenda_link: Scope<Agenda>,
    pub user_info: Rc<Option<UserInfo>>,
}

impl PartialEq for PopupProps {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event && self.user_info == other.user_info
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

        Self {
            comments: None,
            counter: 0,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        if ctx.props().event.eid != old_props.event.eid {
            *self = Component::create(ctx);
        }
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PopupMsg::CommentsLoaded(mut new_comments) => {
                new_comments.sort_by_key(|c| c.downvotes as isize - c.upvotes as isize);
                self.comments = Some(new_comments);
                true
            }
            PopupMsg::SaveColors => {
                let mobile = width() <= 1000;
                let document = window().doc();
                let el = document.get_element_by_id("popup-color-input").unwrap();
                let background_color = el.dyn_into::<HtmlInputElement>().unwrap().value();

                COLORS.set(&ctx.props().event.summary, background_color); 

                // We need to set this so that other events know that they have to refresh
                COLORS_CHANGED.store(true, Ordering::Relaxed);

                if !mobile {
                    ctx.props().agenda_link.send_message(AgendaMsg::Refresh);
                }
                
                true
            }
            PopupMsg::Comment => {
                let el = window().doc().get_element_by_id("comment-textarea-top").unwrap();
                let textarea = el.dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
                let content = textarea.value();
                textarea.set_value("");
                
                let mut comments = self.comments.take().unwrap_or_default();
                comments.push(Comment {
                    cid: self.counter,
                    parent: None,
                    author: UserDesc {
                        uid: ctx.props().user_info.as_ref().as_ref().map(|u| u.uid).unwrap_or(0),
                        email: ctx.props().user_info.as_ref().as_ref().map(|u| u.email.0.split('@').next().unwrap().to_string()).unwrap_or(String::from("unknown")),
                        picture: None,
                    },
                    content: content.clone(),
                    creation_ts: now(),
                    last_edited_ts: now(),
                    upvotes: 1,
                    downvotes: 0,
                    vote: 1,
                });
                self.counter += 1;
                self.comments = Some(comments);

                let eid = ctx.props().event.eid.clone();
                spawn_local(async move {
                    if let Err(e) = update_comment(eid, None, None, content).await {
                        alert(e.to_string());
                    }
                });
                
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick_close = ctx.props().agenda_link.callback(move |_| AgendaMsg::AppMsg(AppMsg::SetPage(Page::Agenda)));
        let event_color = COLORS.get(&ctx.props().event.summary);
        let summary = &ctx.props().event.summary;
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
                    user_info={Rc::clone(&ctx.props().user_info)} />
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
            onclick_close = {onclick_close.clone()},
            onclick_save = {ctx.link().callback(|_| PopupMsg::SaveColors)},
            opt_location = {&opt_location},
            event_color = {event_color.clone()},
            ...
        )
    }
}
