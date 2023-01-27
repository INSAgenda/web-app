use crate::{prelude::*, slider::width};

pub struct Popup {
    friend_counter_folded: bool,
}

pub enum PopupMsg {
    TriggerFriendCounter,
    SaveColors,
}

#[derive(Properties, Clone)]
pub struct PopupProps {
    pub event: RawEvent,
    pub agenda_link: Scope<Agenda>,
    pub friends: Rc<Option<FriendLists>>,
}

impl PartialEq for PopupProps {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event
    }
}

impl Component for Popup {
    type Message = PopupMsg;
    type Properties = PopupProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            friend_counter_folded: true,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
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
            PopupMsg::TriggerFriendCounter => {
                self.friend_counter_folded = !self.friend_counter_folded;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick_close = ctx.props().agenda_link.callback(move |_| AgendaMsg::AppMsg(AppMsg::SetPage(Page::Agenda)));

        // Friend counter
        let friends: Vec<_> = ctx.props().friends.deref().as_ref().map(|friends| {
            friends.friends.iter().filter(|friend| friend.1.matches(&ctx.props().event.group)).map(|f| &f.0).collect()
        }).unwrap_or_default();
        let names = friends.iter().map(|friend| friend.email.trim_end_matches("@insa-rouen.fr")).collect::<Vec<_>>();
        let names_iter = names.iter();
        let friend_count = friends.len();
        let friend_counter_folded = friend_count != 0 && self.friend_counter_folded;
        let friend_counter_unfolded = friend_count != 0 && !self.friend_counter_folded;
        let only_one_friend = friend_count == 1;
        let z_index_iter = 1..friend_count+1;

        let event_color = COLORS.get(&ctx.props().event.summary);
        let summary = &ctx.props().event.summary;
        let name = ctx.props().event.format_name();
        let opt_location = ctx.props().event.format_location();
        template_html!(
            "src/popup/popup.html",
            teachers = {ctx.props().event.teachers.join(", ")},
            time = {ctx.props().event.format_time()},
            name = {&name},
            onclick_close = {onclick_close.clone()},
            onclick_save = {ctx.link().callback(|_| PopupMsg::SaveColors)},
            onclick_fold = {ctx.link().callback(|_| PopupMsg::FoldFriendCounter)},
            opt_location = {&opt_location},
            event_color = {event_color.clone()},
            alt_iter = { names.iter().map(|name| format!("Avatar of {}", name)) },
            picture_iter = { friends.iter().map(|friend| friend.profile_url()) },
            ...
        )
    }
}
