use crate::prelude::*;

lazy_static::lazy_static!{
    pub static ref SETTINGS: SettingStore = {
        let local_storage = window().local_storage().unwrap().unwrap();
        let theme = match local_storage.get_item("setting-theme").unwrap() {
            Some(theme) if theme == "dark" => 0,
            Some(theme) if theme == "light" => 1,
            _ => 2,
        };
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

        SettingStore {
            theme: AtomicUsize::new(theme),
            lang: AtomicUsize::new(lang),
        }
    };
}

pub enum Theme {
    Dark = 0,
    Light,
    System,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    French = 0,
    English,
}

pub struct SettingStore {
    theme: AtomicUsize,
    lang: AtomicUsize,
}

impl SettingStore {
    pub fn theme(&self) -> Theme {
        match self.theme.load(Ordering::Relaxed) {
            0 => Theme::Dark,
            1 => Theme::Light,
            2 => Theme::System,
            _ => unreachable!(),
        }
    }

    fn set_theme(&self, theme: usize) {
        self.theme.store(theme, Ordering::Relaxed);

        let theme = match theme {
            0 => "dark",
            1 => "light",
            2 => "system",
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
}

pub enum Msg {
    Confirm,
    Cancel,
    ThemeChange(usize),
    LogOut,
    LanguageChange(usize),
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
}

impl Component for SettingsPage {
    type Message = Msg;
    type Properties = SettingsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            clone_storage: SettingStore {
                theme: AtomicUsize::new(SETTINGS.theme.load(Ordering::Relaxed)),
                lang: AtomicUsize::new(SETTINGS.lang.load(Ordering::Relaxed)),
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Confirm => {
                // Collect checked groups
                if let Some(user_info) = ctx.props().user_info.as_ref() {
                    let mut available_groups = user_info.available_groups.groups().iter().cloned().collect::<Vec<_>>();
                    available_groups.sort();

                    let document = window().doc();
                    let mut groups = Groups::new();
                    for (i, group) in available_groups.iter().enumerate() {
                        let el = document.get_element_by_id(&format!("group-radio-{}", i)).unwrap();
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
                                Err(e) => alert(format!("Impossible de mettre à jour les groupes : {}", e)),
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
            Msg::ThemeChange(v) => {
                SETTINGS.set_theme(v);

                let theme = match v {
                    0 => "dark",
                    1 => "light",
                    2 => "system",
                    _ => unreachable!(),
                };

                let window = window();
                let doc = window.doc();
                let html = doc.first_element_child().unwrap();
                let storage = window.local_storage().unwrap().unwrap();

                if theme == "system" {
                    storage.set_item("auto-theme", "true").unwrap();
                } else {
                    storage.set_item("auto-theme", "false").unwrap();
                    html.set_attribute("data-theme", theme).unwrap();
                    storage.set_item("setting-theme", theme).unwrap();
                }
                                
                true
            }
            Msg::LogOut => {
                wasm_bindgen_futures::spawn_local(async move {
                    match logout().await{
                        Ok(_) => (),
                        Err(e) => {
                            sentry_report(&e);
                            alert_no_reporting(t("Echec de la déconnexion. Nous avons connaissance de ce problème et travaillons à sa résolution."));
                        },
                    }
                });
                window().location().replace("/login").unwrap();
                false
            }
            Msg::LanguageChange(v) => {
                SETTINGS.set_lang(v);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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

        let group_radio_i_iter = (0..available_groups.len()).map(|i| i.to_string());
        let group_radio_i2_iter = group_radio_i_iter.clone();
        let group_radio_i3_iter = group_radio_i_iter.clone();
        let group_radio_label_iter = available_groups.iter().cloned();
        let group_radio_official_iter = available_groups.iter().map(|g| if official_groups.groups().contains(g) {"(officiel)"} else {""});
        let group_radio_checked_iter = available_groups.iter().map(|g| selected_groups.groups().contains(g));

        let theme_glider_selector = html! {
            <GliderSelector
                values = { vec![t("Sombre"), t("Clair"), t("Système")] }
                on_change = { ctx.link().callback(Msg::ThemeChange) }
                selected = { SETTINGS.theme() as usize } />
        };
        let language_glider_selector = html! {
            <GliderSelector
                values = { vec!["Français", "English"] }
                on_change = { ctx.link().callback(Msg::LanguageChange) }
                selected = { SETTINGS.lang() as usize } />
        };

        template_html!(
            "src/settings/settings.html",
            onclick_rick = {ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Rick))},
            onclick_logout = {ctx.link().callback(move |_| Msg::LogOut)},
            onclick_confirm = {ctx.link().callback(move |_| Msg::Confirm)},
            onclick_delete = {ctx.link().callback(move |_| Msg::Delete)},
            onclick_cancel = {ctx.link().callback(move |_| Msg::Cancel)},
            ...
        )
    }
}
