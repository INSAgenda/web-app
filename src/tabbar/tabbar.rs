use crate::prelude::*;

pub struct TabBar;

#[derive(Clone, Properties)]
pub struct TabBarProps {
    pub app_link: AppLink,
    pub page: Page,
}

impl PartialEq for TabBarProps {
    fn eq(&self, _: &Self) -> bool { true }
}

pub enum TabBarMsg {
}

impl Component for TabBar {
    type Message = TabBarMsg;
    type Properties = TabBarProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let page = &ctx.props().page;
        let onclick_home = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Agenda));
        let home = matches!(page, Page::Agenda);
        let onclick_friends = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Agenda));
        let friends = false;
        let onclick_notifications = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Agenda));
        let notifications = false;
        let onclick_settings = ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Settings));
        let settings = matches!(page, Page::Settings);
        
        template_html!("src/tabbar/tabbar.html", ...)
    }
}
