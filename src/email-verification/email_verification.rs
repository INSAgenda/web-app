use crate::prelude::*;

pub struct EmailVerification {
    state: State,
}

enum State {
    Initial,
    Loading,
    Success,
    Error(ApiError),
}

pub enum EmailVerificationMsg {
    Verify,
    Success,
    Error(ApiError)
}

#[derive(Clone, Properties)]
pub struct EmailVerificationProps {
    pub feature: &'static str,
    pub email: Option<String>,
    pub app_link: AppLink,
}

impl PartialEq for EmailVerificationProps {
    fn eq(&self, other: &Self) -> bool { self.feature == other.feature }
}

impl Component for EmailVerification {
    type Message = EmailVerificationMsg;
    type Properties = EmailVerificationProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: State::Initial,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EmailVerificationMsg::Verify => {
                let link2 = ctx.link().clone();
                spawn_local(async move {
                    match new_confirmation_email().await {
                        Ok(()) => link2.send_message(EmailVerificationMsg::Success),
                        Err(err) => link2.send_message(EmailVerificationMsg::Error(err)),
                    }
                });
                self.state = State::Loading;
                true
            }
            EmailVerificationMsg::Success => {
                self.state = State::Success;
                true
            }
            EmailVerificationMsg::Error(err) => {
                self.state = State::Error(err);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let feature = ctx.props().feature;
        let fmt_feature = match feature {
            "friends" => "d'amis",
            _ => "[unknown]",
        };
        let email = ctx.props().email.as_deref().unwrap_or("[unknown]");

        let success = matches!(self.state, State::Success);
        let is_loading = matches!(self.state, State::Loading);
        let opt_error = match &self.state {
            State::Error(err) => Some(err),
            _ => None,
        };

        template_html!(
            "src/email-verification/email_verification.html",
            onclick_send = {ctx.link().callback(|_| EmailVerificationMsg::Verify)},
            onclick_back = {ctx.props().app_link.callback(move |_| AppMsg::SetPage(match feature {
                "friends" => Page::Friends,
                _ => Page::Agenda,
            }))},
            ...
        )
    }
}
