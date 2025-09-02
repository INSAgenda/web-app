use std::{sync::atomic::AtomicBool, collections::HashSet};
use crate::prelude::*;

fn random_theme() -> Theme {
    let mut bytes = [0u8; 1];
    getrandom::getrandom(&mut bytes).unwrap();
    if bytes[0] <= 150 {
        Theme::Insarcade
    } else {
        Theme::MoyenInsage
    }
}

fn get_groups(ctx: &Context<SettingsPage>, group_list_expanded: bool) -> (Groups, Groups, Vec<String>) {
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

lazy_static::lazy_static!{
    pub static ref SETTINGS: SettingStore = {
        let local_storage = window().local_storage().unwrap().unwrap();
        let (mut theme, randomly_selected) = match local_storage.get_item("setting-theme").unwrap() {
            Some(theme) if theme == "dark" => (Theme::Dark, false),
            Some(theme) if theme == "light" => (Theme::Light, false),
            Some(theme) if theme == "insarcade" => (Theme::Insarcade, false),
            Some(theme) if theme == "moyeninsage" => (Theme::MoyenInsage, false),
            Some(theme) if theme == "random" => (random_theme(), true),
            _ => (Theme::System, false),
        };
        let mut update_theme = randomly_selected;
        if (theme == Theme::Insarcade || theme == Theme::MoyenInsage) && (1712268000..=1712268000+86400).contains(&now()) {
            theme = Theme::Light;
            update_theme = true;
        }

        if update_theme {
            let window = window();
            let doc = window.doc();
            let html = doc.first_element_child().unwrap();
            html.set_attribute("data-theme", theme.as_ref()).unwrap();
        }

        let lang = match local_storage.get_item("setting-lang").unwrap() {
            Some(lang) if lang == "french" => 0,
            Some(lang) if lang == "english" => 1,
            Some(lang) => {alert(format!("Invalid language {lang}")); 0},
            None => {
                let languages = window().navigator().languages();
                let mut lang = None;
                for language in languages.iter() {
                    if let Some(language) = language.as_string() {
                        if language == "fr" || language.starts_with("fr-") {
                            lang = Some(0);
                            break;
                        } else if language == "en" || language.starts_with("en-") {
                            lang = Some(1);
                            break;
                        }
                    }
                }

                lang.unwrap_or(0)
            },
        };
        let calendar = match local_storage.get_item("setting-calendar").unwrap() {
            Some(calendar) if calendar == "gregorian" => 0,
            Some(calendar) if calendar == "republican" => 1,
            _ => 0,
        };

        SettingStore {
            theme: AtomicUsize::new(theme as usize),
            randomly_selected: AtomicBool::new(randomly_selected),
            lang: AtomicUsize::new(lang),
            calendar: AtomicUsize::new(calendar),
        }
    };
}

#[derive(PartialEq, Eq, Debug)]
pub enum Theme {
    Dark = 0,
    Light,
    System,
    Insarcade,
    MoyenInsage,
    Random,
}

impl AsRef<str> for Theme {
    fn as_ref(&self) -> &str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::System => "system",
            Theme::Insarcade => "insarcade",
            Theme::MoyenInsage => "moyeninsage",
            Theme::Random => "random",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    French = 0,
    English,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CalendarKind {
    Gregorian = 0,
    Republican,
}

pub struct SettingStore {
    theme: AtomicUsize,
    randomly_selected: AtomicBool,
    lang: AtomicUsize,
    calendar: AtomicUsize,
}

impl SettingStore {
    pub fn theme(&self) -> Theme {
        match self.theme.load(Ordering::Relaxed) {
            0 => Theme::Dark,
            1 => Theme::Light,
            2 => Theme::System,
            3 => Theme::Insarcade,
            4 => Theme::MoyenInsage,
            5 => Theme::Random,
            _ => unreachable!(),
        }
    }

    fn real_theme(&self) -> Theme {
        if self.randomly_selected.load(Ordering::Relaxed) {
            return Theme::Random;
        }
        self.theme()
    }

    fn randomly_selected(&self) -> bool {
        self.randomly_selected.load(Ordering::Relaxed)
    }

    fn set_randomly_selected(&self, value: bool) {
        self.randomly_selected.store(value, Ordering::Relaxed)
    }

    fn set_theme(&self, theme: usize) {
        self.theme.store(theme, Ordering::Relaxed);

        let theme = match theme {
            0 => "dark",
            1 => "light",
            2 => "system",
            3 => "insarcade",
            4 => "moyeninsage",
            5 => "random",
            _ => unreachable!(),
        };

        let storage = window().local_storage().unwrap().unwrap();
        storage.set_item("setting-theme", theme).unwrap();
    }

    pub fn lang(&self) -> Lang {
        match self.lang.load(Ordering::Relaxed) {
            0 => Lang::French,
            1 => Lang::English,
            _ => unreachable!(),
        }
    }

    pub fn locale(&self) -> &str {
        match self.lang() {
            Lang::French => "fr",
            Lang::English => "en",
        }
    }

    fn set_lang(&self, lang: usize) {
        self.lang.store(lang, Ordering::Relaxed);

        let lang = match lang {
            0 => "french",
            1 => "english",
            _ => unreachable!(),
        };

        let storage = window().local_storage().unwrap().unwrap();
        storage.set_item("setting-lang", lang).unwrap();
    }

    pub fn calendar(&self) -> CalendarKind {
        match self.calendar.load(Ordering::Relaxed) {
            0 => CalendarKind::Gregorian,
            1 => CalendarKind::Republican,
            _ => unreachable!(),
        }
    }

    pub fn set_calendar(&self, calendar: usize) {
        self.calendar.store(calendar, Ordering::Relaxed);

        let calendar = match calendar {
            0 => "gregorian",
            1 => "republican",
            _ => unreachable!(),
        };

        let storage = window().local_storage().unwrap().unwrap();
        storage.set_item("setting-calendar", calendar).unwrap();
    }
}

pub enum Msg {
    Confirm,
    Cancel,
    ThemeChange(usize),
    LogOut,
    LanguageChange(usize),
    CalendarChange(usize),
    RegenerateToken,
    CopyIcs,
    GroupListToggle,
}

#[derive(Properties, Clone)]
pub struct SettingsProps {
    pub app_link: Scope<App>,
    pub user_info: Rc<Option<UserInfo>>,
}

impl PartialEq for SettingsProps {
    fn eq(&self, other: &Self) -> bool { 
        self.user_info == other.user_info
    }
}

pub struct SettingsPage {
    clone_storage: SettingStore,
    group_list_expanded: bool,
}

impl Component for SettingsPage {
    type Message = Msg;
    type Properties = SettingsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut group_list_expanded = false;
        if let Some(user_info) = ctx.props().user_info.as_ref() {
            let selected_groups_count = user_info.groups.groups().len();
            let official_groups_count = user_info.official_groups.groups().len();
            if selected_groups_count + official_groups_count == 0 {
                group_list_expanded = true;
            }
        }

        Self {
            clone_storage: SettingStore {
                theme: AtomicUsize::new(SETTINGS.theme.load(Ordering::Relaxed)),
                randomly_selected: AtomicBool::new(SETTINGS.randomly_selected()),
                lang: AtomicUsize::new(SETTINGS.lang.load(Ordering::Relaxed)),
                calendar: AtomicUsize::new(SETTINGS.calendar.load(Ordering::Relaxed)),
            },
            group_list_expanded,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Confirm => {
                if let Some(user_info) = ctx.props().user_info.as_ref() {
                    let (_, _, shown_groups) = get_groups(ctx, self.group_list_expanded);

                    // Get the groups selected by the user
                    let document = window().doc();
                    let mut groups = Groups::new();
                    for (i, group) in shown_groups.iter().enumerate() {
                        let Some(el) = document.get_element_by_id(&format!("group-radio-{i}")) else {continue};
                        let el = el.dyn_into::<HtmlInputElement>().unwrap();
                        if el.checked() {
                            groups.insert(group);
                        }
                    }
                    
                    // Update the groups both in backend and frontend
                    if groups != user_info.groups {
                        let app_link = ctx.props().app_link.clone();
                        let user_info = user_info.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match api_post(groups.clone(), "set-groups").await {
                                Ok(()) => app_link.send_message(AppMsg::UserInfoSuccess(UserInfo {groups, ..user_info})),
                                Err(e) => alert(format!("Impossible de mettre à jour les groupes : {e}")),
                            }
                        })
                    }
                }

                ctx.props().app_link.send_message(AppMsg::SetPage(Page::Agenda));
                false
            }
            Msg::Cancel => {
                ctx.props().app_link.send_message(AppMsg::SetPage(Page::Agenda));
                SETTINGS.set_theme(self.clone_storage.theme.load(Ordering::Relaxed));
                SETTINGS.set_lang(self.clone_storage.lang.load(Ordering::Relaxed));
                false
            }
            Msg::ThemeChange(v_to_store) => {
                let mut v_to_display = v_to_store;
                if v_to_store == 5 {
                    v_to_display = random_theme() as usize;
                    SETTINGS.set_randomly_selected(true);
                }
                SETTINGS.set_theme(v_to_display);

                let theme_to_store = match v_to_store {
                    0 => "dark",
                    1 => "light",
                    2 => "system",
                    3 => "insarcade",
                    4 => "moyeninsage",
                    5 => "random",
                    _ => unreachable!(),
                };

                let theme_to_display = match v_to_display {
                    0 => "dark",
                    1 => "light",
                    2 => "system",
                    3 => "insarcade",
                    4 => "moyeninsage",
                    _ => unreachable!(),
                };

                let window = window();
                let doc = window.doc();
                let html = doc.first_element_child().unwrap();
                let storage = window.local_storage().unwrap().unwrap();

                if theme_to_store == "system" {
                    storage.set_item("auto-theme", "true").unwrap();
                } else {
                    storage.set_item("auto-theme", "false").unwrap();
                    html.set_attribute("data-theme", theme_to_display).unwrap();
                    storage.set_item("setting-theme", theme_to_store).unwrap();
                }
                
                true
            }
            Msg::LogOut => {
                // Clear local storage but themes
                let window = window();
                let local_storage = window.local_storage().unwrap().unwrap();
                let theme = local_storage.get("setting-theme").unwrap();
                let auto = local_storage.get("auto-theme").unwrap();
                local_storage.clear().unwrap();
                if let Some(theme) = theme {
                    local_storage.set("setting-theme", &theme).unwrap();
                }
                if let Some(auto) = auto {
                    local_storage.set("auto-theme", &auto).unwrap();
                }

                window.location().replace("https://auth.dera.page/logout").unwrap();
                false
            }
            Msg::LanguageChange(v) => {
                SETTINGS.set_lang(v);
                true
            }
            Msg::CalendarChange(v) => {
                SETTINGS.set_calendar(v);
                true
            }
            Msg::GroupListToggle => {
                self.group_list_expanded = !self.group_list_expanded;
                true
            }
            Msg::RegenerateToken => {
                let app_link = ctx.props().app_link.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = api_post::<()>((), "regenerate-token").await {
                        alert(format!("Impossible de régénérer le token : {e}"));
                    } else {
                        // Force refresh of user info to update token
                        <UserInfo as CachedData>::refresh(app_link);
                    }
                });
                true
            }
            Msg::CopyIcs => {
                let document = window().doc();
                let value = document
                    .get_element_by_id("ics-url")
                    .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
                    .map(|i| i.value())
                    .unwrap_or_default();

                let nav = window().navigator();
                if let Ok(clipboard) = Reflect::get(&nav, &JsValue::from_str("clipboard")) {
                    if let Ok(write_text) = Reflect::get(&clipboard, &JsValue::from_str("writeText")) {
                        if let Ok(f) = write_text.dyn_into::<js_sys::Function>() {
                            let _ = f.call1(&clipboard, &JsValue::from_str(&value));
                            return false;
                        }
                    }
                }

                // Fallback: select text so user can Cmd+C
                if let Some(input) = document.get_element_by_id("ics-url") {
                    if let Ok(input) = input.dyn_into::<HtmlInputElement>() {
                        input.select();
                    }
                }
                false
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

        let theme_glider_selector = html! {
            <GliderSelector
                values = { vec![t("Sombre"), t("Clair"), t("Système"), "Ins'arcade", "Moyen InsAge", "Aléatoire"] }
                on_change = { ctx.link().callback(Msg::ThemeChange) }
                selected = { SETTINGS.real_theme() as usize } />
        };
        let language_glider_selector = html! {
            <GliderSelector
                values = { vec!["Français", "English"] }
                on_change = { ctx.link().callback(Msg::LanguageChange) }
                selected = { SETTINGS.lang() as usize } />
        };
        let calendar_glider_selector = html! {
            <GliderSelector
                values = { vec!["Gregorien", "Républicain"] }
                on_change = { ctx.link().callback(Msg::CalendarChange) }
                selected = { SETTINGS.calendar() as usize } />
        };
        let token = if let Some(user_info) = ctx.props().user_info.as_ref() { user_info.token.clone() } else { "".to_string() };

        // Build ICS absolute URL
        let location = window().location();
        let origin = location.origin().unwrap_or_else(|_| "".into());
        let ics_url = if token.is_empty() { String::new() } else { format!("{origin}/api/ics?token={token}") };

        template_html!(
            "src/settings/settings.html",
            onclick_rick = {ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Rick))},
            onclick_logout = {ctx.link().callback(move |_| Msg::LogOut)},
            onclick_confirm = {ctx.link().callback(move |_| Msg::Confirm)},
            onclick_delete = {ctx.link().callback(move |_| Msg::Delete)},
            onclick_cancel = {ctx.link().callback(move |_| Msg::Cancel)},
            onclick_group_list_toggle = {ctx.link().callback(|_| Msg::GroupListToggle)},
            onclick_copy_ics = {ctx.link().callback(|_| Msg::CopyIcs)},
            onclick_regenerate_token = {ctx.link().callback(|_| Msg::RegenerateToken)},
            republican = {SETTINGS.calendar() == CalendarKind::Republican},
            ics_url = {ics_url},
            ics_open_url = {ics_open_url},
            ...
        )
    }
}
