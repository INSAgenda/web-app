use std::collections::HashSet;
use crate::prelude::*;

fn get_groups(ctx: &Context<OnboardingPage>, group_list_expanded: bool) -> (Groups, Groups, Vec<String>) {
    // Compute variable messages
    let mut official_groups = Groups::new();
    let mut selected_groups = Groups::new();
    let mut available_groups = Vec::new();
    if let Some(user_info) = ctx.props().user_info.as_ref() {
        official_groups = user_info.official_groups.clone();
        selected_groups = user_info.groups.clone();
        available_groups = user_info.available_groups.groups().iter().cloned().collect::<Vec<_>>();
        available_groups.sort();
    }

    // If the group list is not expanded, only show recommended groups
    let mut shown_groups = available_groups;
    if !group_list_expanded {
        let recommended_prefixes = selected_groups
            .groups()
            .iter()
            .chain(official_groups.groups().iter())
            .map(|g| g.split('-').next().unwrap_or(g))
            .collect::<HashSet<_>>();
        shown_groups.retain(|g| recommended_prefixes.iter().any(|p| g.starts_with(p)));
    }

    (official_groups, selected_groups, shown_groups)
}

pub enum Msg {
    Complete,
    Skip,
    GroupListToggle,
}

#[derive(Properties, Clone)]
pub struct OnboardingProps {
    pub app_link: Scope<App>,
    pub user_info: Rc<Option<UserInfo>>,
}

impl PartialEq for OnboardingProps {
    fn eq(&self, other: &Self) -> bool { 
        self.user_info == other.user_info
    }
}

pub struct OnboardingPage {
    group_list_expanded: bool,
}

impl Component for OnboardingPage {
    type Message = Msg;
    type Properties = OnboardingProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (_, _, shown_groups) = get_groups(ctx, false);
        Self {
            group_list_expanded: shown_groups.is_empty()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Complete => {
                if let Some(user_info) = ctx.props().user_info.as_ref() {
                    let (_, _, shown_groups) = get_groups(ctx, self.group_list_expanded);

                    // Get the groups selected by the user
                    let document = window().doc();
                    let mut groups = Groups::new();
                    for (i, group) in shown_groups.iter().enumerate() {
                        let Some(el) = document.get_element_by_id(&format!("onboarding-group-radio-{i}")) else {continue};
                        let el = el.dyn_into::<HtmlInputElement>().unwrap();
                        if el.checked() {
                            groups.insert(group);
                        }
                    }
                    
                    // Mark user as onboarded and update groups
                    let app_link = ctx.props().app_link.clone();
                    let mut updated_user_info = user_info.clone();
                    updated_user_info.onboarded = true;
                    updated_user_info.groups = groups.clone();
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        // Update groups first
                        match api_post(groups.clone(), "set-groups").await {
                            Ok(()) => {
                                app_link.send_message(AppMsg::UserInfoSuccess(updated_user_info));
                                app_link.send_message(AppMsg::SetPage(Page::Agenda));
                            },
                            Err(e) => alert(format!("Impossible de mettre à jour les groupes : {e}")),
                        }
                    });
                }
                false
            }
            Msg::Skip => {
                if let Some(user_info) = ctx.props().user_info.as_ref() {
                    // Just mark as onboarded without changing groups
                    let app_link = ctx.props().app_link.clone();
                    let mut updated_user_info = user_info.clone();
                    updated_user_info.onboarded = true;
                    
                    app_link.send_message(AppMsg::UserInfoSuccess(updated_user_info));
                    app_link.send_message(AppMsg::SetPage(Page::Agenda));
                }
                false
            }
            Msg::GroupListToggle => {
                self.group_list_expanded = !self.group_list_expanded;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (official_groups, selected_groups, shown_groups) = get_groups(ctx, self.group_list_expanded);

        // Produce iterators for the template
        let group_radio_i_iter = (0..shown_groups.len()).map(|i| i.to_string());
        let group_radio_i2_iter = group_radio_i_iter.clone();
        let group_radio_i3_iter = group_radio_i_iter.clone();
        let group_radio_label_iter = shown_groups.iter().cloned();
        let group_radio_official_iter = shown_groups.iter().map(|g| if official_groups.groups().contains(g) {"(officiel)"} else {""});
        let group_radio_checked_iter = shown_groups.iter().map(|g| selected_groups.groups().contains(g));
        let group_list_expanded = self.group_list_expanded;

        template_html!(
            "src/onboarding/onboarding.html",
            onclick_complete = {ctx.link().callback(move |_| Msg::Complete)},
            onclick_skip = {ctx.link().callback(move |_| Msg::Skip)},
            onclick_group_list_toggle = {ctx.link().callback(|_| Msg::GroupListToggle)},
            ...
        )
    }
}
