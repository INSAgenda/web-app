use crate::prelude::*;

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

pub struct CommentComp {
    vote: i8,
}

impl Component for CommentComp {
    type Message = CommentMsg;
    type Properties = CommentProps;

    fn create(ctx: &Context<Self>) -> Self {
        let comment = ctx.props().comments.iter().find(|comment| comment.cid == ctx.props().cid).unwrap();
        Self {
            vote: comment.vote,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CommentMsg::Upvote => {
                if self.vote == 1 {
                    self.vote = 0;
                } else {
                    self.vote = 1;
                }
            }
            CommentMsg::Downvote => {
                if self.vote == -1 {
                    self.vote = 0;
                } else {
                    self.vote = -1;
                }
            }
            CommentMsg::Reply => {

            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.vote = ctx.props().comments.iter().find(|comment| comment.cid == ctx.props().cid).unwrap().vote;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let comment = ctx.props().comments.iter().find(|comment| comment.cid == ctx.props().cid).unwrap();
        let author_avatar = format!("https://api.dicebear.com/5.x/adventurer/svg?seed={}", comment.author.uid);
        let author_name = comment.author.get_username();
        let time_diff = now() - comment.creation_ts;
        let time = format_time_diff(time_diff);
        let content = &comment.content;
        let score = comment.score - comment.vote as i64 + self.vote as i64;
        let upvote_class = match self.vote {
            1 => "comment-upvoted",
            -1 => "comment-downvoted",
            _ => "comment-not-voted",
        };

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
