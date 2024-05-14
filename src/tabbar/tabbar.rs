use crate::prelude::*;

pub struct TabBar {}

#[derive(Clone, Properties)]
pub struct TabBarProps {
    pub app_link: AppLink,
    pub bait_points: (bool, bool, bool, bool, bool),
    #[prop_or(false)]
    pub others_disabled: bool,
    pub page: Page,
}

impl PartialEq for TabBarProps {
    fn eq(&self, other: &Self) -> bool { self.page.eq(&other.page) && self.bait_points.eq(&other.bait_points) && self.others_disabled.eq(&other.others_disabled) }
}

pub enum TabBarMsg {}

impl Component for TabBar {
    type Message = TabBarMsg;
    type Properties = TabBarProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let page = &ctx.props().page;
        
        let onclick_home = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Agenda));
        let mut home_classes = String::from(if matches!(page, Page::Agenda) {"tabbar-selected"} else {"tabbar-not-selected"});
        if ctx.props().bait_points.0 { home_classes.push_str(" tabbar-with-bait"); }
        if ctx.props().others_disabled { home_classes.push_str(" tabbar-disabled"); }

        let onclick_friends = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Friends));
        let mut friends_classes = String::from(if matches!(page, Page::Friends | Page::FriendAgenda { .. }) {"tabbar-selected"} else {"tabbar-not-selected"});
        if ctx.props().bait_points.1 { friends_classes.push_str(" tabbar-with-bait"); }
        if ctx.props().others_disabled { friends_classes.push_str(" tabbar-disabled"); }

        let onclick_mastodon = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Mastodon));
        let mut mastodon_classes = String::from(if matches!(page, Page::Mastodon) {"tabbar-selected"} else {"tabbar-not-selected"});
        if ctx.props().bait_points.2 { mastodon_classes.push_str(" tabbar-with-bait"); }
        if ctx.props().others_disabled { mastodon_classes.push_str(" tabbar-disabled"); }

        let onclick_settings = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Settings));
        let mut settings_classes = String::from(if matches!(page, Page::Settings) {"tabbar-selected"} else {"tabbar-not-selected"});
        if ctx.props().bait_points.3 { settings_classes.push_str(" tabbar-with-bait"); }
        if ctx.props().others_disabled { settings_classes.push_str(" tabbar-disabled"); }

        let onclick_pixelwar = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::PixelWar));
        let mut pixelwar_classes = String::from(if matches!(page, Page::PixelWar) {"tabbar-selected"} else {"tabbar-not-selected"});
        if ctx.props().bait_points.3 { pixelwar_classes.push_str(" tabbar-with-bait"); }

        template_html!("src/tabbar/tabbar.html", ...)
    }
}
