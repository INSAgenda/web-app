use crate::prelude::*;

pub struct TabBar;

#[derive(Clone, Properties)]
pub struct TabBarProps {
    pub app_link: AppLink,
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
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let page = &ctx.props().page;
        let onclick_home = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Agenda));
        let home_class = if matches!(page, Page::Agenda) {"tabbar-selected"} else {"tabbar-not-selected"};
        let onclick_friends = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Friends));
        let friends_class = if matches!(page, Page::Friends) {"tabbar-selected"} else {"tabbar-not-selected"};
        let onclick_notifications = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Agenda));
        let notifications_class = "tabbar-not-selected";
        let onclick_settings = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Settings));
        let settings_class = if matches!(page, Page::Settings | Page::ChangeEmail | Page::ChangeGroup | Page::ChangePassword) {"tabbar-selected"} else {"tabbar-not-selected"};
        
        template_html!("src/tabbar/tabbar.html", ...)
    }
}
