use crate::prelude::*;

pub struct CommentComp;

pub enum CommentMsg {
    Upvote,
    Downvote,
    Reply,
}

impl Component for CommentComp {
    type Message = CommentMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let author_avatar = "https://api.dicebear.com/5.x/adventurer/svg?seed=Molly";
        let author_name = "John Doe";
        let time = "1 hour ago";
        let content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec auctor, nisl eget ultricies lacinia, nisl nisl aliquet nisl, nec aliquet nisl nisl sit amet nisl.";
        let score = 5;
        let upvoted = true;
        let downvoted = true;

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
