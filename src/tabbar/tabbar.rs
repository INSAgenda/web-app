use web_sys::{Storage, HtmlAudioElement};

use crate::prelude::*;

pub struct TabBar {
    audio_element: HtmlAudioElement,
}

#[derive(Clone, Properties)]
pub struct TabBarProps {
    pub app_link: AppLink,
    pub bait_points: (bool, bool, bool, bool),
    pub page: Page,
}

impl PartialEq for TabBarProps {
    fn eq(&self, other: &Self) -> bool { self.page.eq(&other.page) }
}

pub enum TabBarMsg {
}

impl Component for TabBar {
    type Message = TabBarMsg;
    type Properties = TabBarProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let storage = CollectedGifts::from_local_storage();

        let audio_element = HtmlAudioElement::new_with_src("/agenda/assets/happy-santa.mp3").unwrap();
        let day22_collected = storage.is_collected(21);
        if day22_collected {
            audio_element.set_autoplay(true);
            audio_element.set_loop(true)
        }

        Self {
            audio_element,
        }
    }


    fn view(&self, ctx: &Context<Self>) -> Html {
        let page = &ctx.props().page;
        
        let onclick_home = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Agenda));
        let mut home_classes = String::from(if matches!(page, Page::Agenda) {"tabbar-selected"} else {"tabbar-not-selected"});
        if ctx.props().bait_points.0 { home_classes.push_str(" tabbar-with-bait"); }

        let onclick_friends = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Friends));
        let mut friends_classes = String::from(if matches!(page, Page::Friends | Page::FriendAgenda { .. }) {"tabbar-selected"} else {"tabbar-not-selected"});
        if ctx.props().bait_points.1 { friends_classes.push_str(" tabbar-with-bait"); }

        let onclick_notifications = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Notifications));
        let mut notifications_classes = String::from(if matches!(page, Page::Notifications) {"tabbar-selected"} else {"tabbar-not-selected"});
        if ctx.props().bait_points.2 { notifications_classes.push_str(" tabbar-with-bait"); }

        let onclick_settings = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Settings));
        let mut settings_classes = String::from(if matches!(page, Page::Settings) {"tabbar-selected"} else {"tabbar-not-selected"});
        if ctx.props().bait_points.3 { settings_classes.push_str(" tabbar-with-bait"); }

        let storage: CollectedGifts = CollectedGifts::from_local_storage();
        let day7_collected = storage.is_collected(6);
        let day22_collected = storage.is_collected(21);

        template_html!("src/tabbar/tabbar.html", ...)
    }
}
