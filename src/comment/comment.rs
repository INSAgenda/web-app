use crate::prelude::*;

pub struct CommentComp;

#[derive(Properties, Clone, PartialEq)]
pub struct CommentProps {
    pub comments: Rc<Vec<Comment>>,
    pub cid: u64,
}

pub enum CommentMsg {
    Upvote,
    Downvote,
    Reply,
}

impl Component for CommentComp {
    type Message = CommentMsg;
    type Properties = CommentProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let comment = ctx.props().comments.iter().find(|comment| comment.cid == ctx.props().cid).unwrap();
        let author_avatar = format!("https://api.dicebear.com/5.x/adventurer/svg?seed={}", comment.author.uid);
        let author_name = comment.author.get_username();
        let time_diff = now() - comment.creation_ts;
        let time = format_time_diff(time_diff);
        let content = &comment.content;
        let score = comment.score;
        let upvoted = comment.vote == 1;
        let downvoted = comment.vote == -1;

        let onclick_upvote = ctx.link().callback(|_| CommentMsg::Upvote);
        let onclick_downvote = ctx.link().callback(|_| CommentMsg::Downvote);
        let onclick_reply = ctx.link().callback(|_| CommentMsg::Reply);

        let children = ctx.props().comments.iter().filter(|child| child.parent == Some(comment.cid)).map(|child| {
            html! {
                <CommentComp comments={Rc::clone(&ctx.props().comments)} cid={child.cid} />
            }
        }).collect::<Html>();

        template_html!(
            "src/comment/comment.html",
            ...
        )
    }
}
