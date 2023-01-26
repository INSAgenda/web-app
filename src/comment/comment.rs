use crate::prelude::*;

pub struct CommentComp;

#[derive(Properties, Clone, PartialEq)]
pub struct CommentProps {
    pub comment: Comment,
    pub children: Vec<Comment>,
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
        let author_avatar = format!("https://api.dicebear.com/5.x/adventurer/svg?seed={}", ctx.props().comment.author.uid);
        let author_name = ctx.props().comment.author.get_username();
        let time_diff = now() - ctx.props().comment.creation_ts;
        let time = format_time_diff(time_diff);
        let content = &ctx.props().comment.content;
        let score = ctx.props().comment.score;
        let upvoted = ctx.props().comment.vote == 1;
        let downvoted = ctx.props().comment.vote == -1;

        let onclick_upvote = ctx.link().callback(|_| CommentMsg::Upvote);
        let onclick_downvote = ctx.link().callback(|_| CommentMsg::Downvote);
        let onclick_reply = ctx.link().callback(|_| CommentMsg::Reply);

        let children: Vec<String> = Vec::new();

        template_html!(
            "src/comment/comment.html",
            children_iter = {children.iter()},
            ...
        )
    }
}
