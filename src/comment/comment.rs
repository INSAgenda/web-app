use crate::prelude::*;

#[derive(Properties, Clone)]
pub struct CommentProps {
    pub eid: Rc<String>,
    pub comments: Rc<Vec<Comment>>,
    pub cid: u64,
    pub user_info: Rc<Option<UserInfo>>,
    pub popup_link: PopupLink,
}

impl PartialEq for CommentProps {
    fn eq(&self, other: &Self) -> bool {
        self.eid == other.eid && self.comments == other.comments && self.cid == other.cid && self.user_info == other.user_info
    }
}

pub enum CommentMsg {
    Upvote,
    Downvote,
    StartReply,
    SubmitReply,
    StartEdit,
    SubmitEdit,
    Report,
    Delete,
}

pub struct CommentComp {
    vote: i8,
    replying: bool,
    editing: bool,
}

impl Component for CommentComp {
    type Message = CommentMsg;
    type Properties = CommentProps;

    fn create(ctx: &Context<Self>) -> Self {
        let comment = ctx.props().comments.iter().find(|comment| comment.cid == ctx.props().cid).unwrap();
        Self {
            vote: comment.vote,
            replying: false,
            editing: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CommentMsg::Upvote => {
                if self.vote == 1 {
                    self.vote = 0;
                } else {
                    self.vote = 1;
                }
                let vote = self.vote;
                let eid = ctx.props().eid.to_string();
                let cid = ctx.props().cid;
                spawn_local(async move {
                    if let Err(e) = update_vote(eid, vote, cid).await {
                        alert(e.to_string());
                    }
                });
            }
            CommentMsg::Downvote => {
                if self.vote == -1 {
                    self.vote = 0;
                } else {
                    self.vote = -1;
                }
                let vote = self.vote;
                let eid = ctx.props().eid.to_string();
                let cid = ctx.props().cid;
                spawn_local(async move {
                    if let Err(e) = update_vote(eid, vote, cid).await {
                        alert(e.to_string());
                    }
                });
            }
            CommentMsg::StartReply => {
                self.replying = !self.replying;
            }
            CommentMsg::StartEdit => {
                self.editing = !self.editing;
                if self.editing {
                    let el = window().doc().get_element_by_id(&format!("comment-textarea-{}", ctx.props().cid)).unwrap();
                    let textarea = el.dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
                    textarea.set_value(&ctx.props().comments.iter().find(|comment| comment.cid == ctx.props().cid).unwrap().content);
                }
            }
            CommentMsg::SubmitReply => {
                let el = window().doc().get_element_by_id(&format!("reply-textarea-{}", ctx.props().cid)).unwrap();
                let textarea = el.dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
                let content = textarea.value();

                self.replying = false;

                let eid = ctx.props().eid.to_string();
                let parent_id = ctx.props().cid;
                let popup_link = ctx.props().popup_link.clone();
                spawn_local(async move {
                    if let Err(e) = update_comment(eid, None, Some(parent_id), content).await {
                        alert(e.to_string());
                    }
                    popup_link.send_message(PopupMsg::ReloadComments);
                });
            }
            CommentMsg::SubmitEdit => {
                let el = window().doc().get_element_by_id(&format!("comment-textarea-{}", ctx.props().cid)).unwrap();
                let textarea = el.dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
                let content = textarea.value();
                
                if content.is_empty() {
                    return false;
                }

                self.editing = false;

                let eid = ctx.props().eid.to_string();
                let cid = ctx.props().cid;
                let popup_link = ctx.props().popup_link.clone();
                spawn_local(async move {
                    if let Err(e) = update_comment(eid, Some(cid), None, content).await {
                        alert(e.to_string());
                    }
                    popup_link.send_message(PopupMsg::ReloadComments);
                });
            }
            CommentMsg::Report => {
                let author_uid = ctx.props().comments.iter().find(|comment| comment.cid == ctx.props().cid).map(|c| c.author.uid).unwrap_or_default();
                if author_uid == 8384614791391388193 || author_uid == 2228748929683190697 {
                    ctx.props().popup_link.send_message(PopupMsg::AppMsg(AppMsg::SetPage(Page::Rick)));
                    return false;
                }
                let url_to_open = format!("mailto:simon.girard@insa-rouen.fr?subject=Report%20de%20commentaire%20({})", ctx.props().cid);
                web_sys::window().unwrap().open_with_url(&url_to_open).unwrap();
            }
            CommentMsg::Delete => {
                let eid = ctx.props().eid.to_string();
                let cid = ctx.props().cid;
                let popup_link = ctx.props().popup_link.clone();
                spawn_local(async move {
                    match api_delete(format!("comment?eid={eid}&cid={cid}")).await {
                        Ok(()) => (),
                        Err(e) => alert(e.to_string()), 
                    }
                    popup_link.send_message(PopupMsg::ReloadComments);
                });
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.vote = ctx.props().comments.iter().find(|comment| comment.cid == ctx.props().cid).unwrap().vote;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let comments = Rc::clone(&ctx.props().comments);
        let comment = match comments.iter().find(|comment| comment.cid == ctx.props().cid) {
            Some(comment) => comment,
            None => return html!(),
        };
        
        let cid = comment.cid;
        let author_avatar = format!("https://api.dicebear.com/5.x/identicon/svg?seed={}", comment.author.uid);
        let author_name = comment.author.get_username();
        let time_diff = now() - comment.creation_ts;
        let time = format_time_diff(time_diff);
        let modified = comment.last_edited_ts > comment.creation_ts;
        let score = comment.upvotes as i64 - comment.downvotes as i64 - comment.vote as i64 + self.vote as i64;
        let upvote_class = match self.vote {
            1 => "comment-upvoted",
            -1 => "comment-downvoted",
            _ => "comment-not-voted",
        };
        let replying = self.replying;
        let editing = self.editing;

        let onclick_upvote = ctx.link().callback(|_| CommentMsg::Upvote);
        let onclick_downvote = ctx.link().callback(|_| CommentMsg::Downvote);
        let onclick_reply = ctx.link().callback(|_| CommentMsg::StartReply);
        let onclick_reply_cancel = onclick_reply.clone();
        let onclick_reply_submit = ctx.link().callback(|_| CommentMsg::SubmitReply);
        let onclick_edit = ctx.link().callback(|_| CommentMsg::StartEdit);
        let onclick_edit_cancel = onclick_edit.clone();
        let onclick_edit_submit = ctx.link().callback(|_| CommentMsg::SubmitEdit);
        let onclick_report = ctx.link().callback(|_| CommentMsg::Report);
        let onclick_delete = ctx.link().callback(move |_| CommentMsg::Delete);

        let children = comments.iter().filter(|child| child.parent == Some(comment.cid)).map(|child| {
            html! {
                <CommentComp
                    eid={Rc::clone(&ctx.props().eid)}
                    comments={Rc::clone(&comments)}
                    cid={child.cid}
                    user_info={Rc::clone(&ctx.props().user_info)}
                    popup_link={ctx.props().popup_link.clone()} />
            }
        }).collect::<Html>();

        let self_uid = ctx.props().user_info.as_ref().as_ref().map(|u| u.uid).unwrap_or(0);
        let is_author = comment.author.uid == self_uid;
        let self_avatar = format!("https://api.dicebear.com/5.x/identicon/svg?seed={self_uid}");
        let self_name = ctx.props().user_info.as_ref().as_ref().map(|u| u.email.0.split('@').next().unwrap().to_string()).unwrap_or(String::from("inconnu"));

        template_html!(
            "src/comment/comment.html",
            content = { comment.content.clone() },
            ...
        )
    }
}
