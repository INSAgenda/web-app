use crate::prelude::*;

lazy_static::lazy_static!{
    pub static ref SETTINGS: SettingStore = {
        let local_storage = window().local_storage().unwrap().unwrap();
        let theme = match local_storage.get_item("setting-theme").unwrap() {
            Some(theme) if theme == "dark" => 0,
            Some(theme) if theme == "light" => 1,
            _ => 2,
        };
        let building_naming = match local_storage.get_item("setting-building-naming").unwrap() {
            Some(building_naming) if building_naming == "short" => 0,
            Some(building_naming) if building_naming == "long" => 1,
            Some(building_naming) => {alert(format!("Invalid building naming {building_naming}")); 0},
            None => 0,
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
            building_naming: AtomicUsize::new(building_naming),
            theme: AtomicUsize::new(theme),
            lang: AtomicUsize::new(lang),
        }
    };
}

pub enum BuildingNaming {
    Short = 0,
    Long,
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
    building_naming: AtomicUsize,
    theme: AtomicUsize,
    lang: AtomicUsize,
}

impl SettingStore {
    pub fn building_naming(&self) -> BuildingNaming {
        match self.building_naming.load(Ordering::Relaxed) {
            0 => BuildingNaming::Short,
            1 => BuildingNaming::Long,
            _ => unreachable!(),
        }
    }

    fn set_building_naming(&self, building_naming: usize) {
        self.building_naming.store(building_naming, Ordering::Relaxed);

        let building_naming = match building_naming {
            0 => "short",
            1 => "long",
            _ => unreachable!(),
        };

        let storage = window().local_storage().unwrap().unwrap();
        storage.set_item("setting-building-naming", building_naming).unwrap();
    }

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
                building_naming: AtomicUsize::new(SETTINGS.building_naming.load(Ordering::Relaxed)),
                theme: AtomicUsize::new(SETTINGS.theme.load(Ordering::Relaxed)),
                lang: AtomicUsize::new(SETTINGS.lang.load(Ordering::Relaxed)),
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Confirm => {
                ctx.props().app_link.send_message(AppMsg::SetPage(Page::Agenda));
                false
            }
            Msg::Cancel => {
                ctx.props().app_link.send_message(AppMsg::SetPage(Page::Agenda));
                SETTINGS.set_building_naming(self.clone_storage.building_naming.load(Ordering::Relaxed));
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
        let mut email_not_verified = false;
        let mut email = String::from(t("[inconnue]"));
        let mut formatted_group = String::from(t("[inconnu]"));
        let mut last_password_mod_str = String::from(t("[indisponible]"));
        if let Some(user_info) = ctx.props().user_info.as_ref() {
            if !user_info.email.1 {
                email_not_verified = true;
            }
            email = user_info.email.0.to_owned();
            if let Some(last_password_mod) = user_info.last_password_mod {
                let now = (js_sys::Date::new_0().get_time() / 1000.0) as i64;
                let diff = now - last_password_mod;
                let words = [["secondes", "minutes", "heures", "jours", "semaines", "mois", "années"], ["seconds ago", "minutes ago", "hours ago", "days ago", "weeks ago", "months ago", "years ago"]];
                let i = if SETTINGS.lang() == Lang::French { 0 } else { 1 };
                last_password_mod_str = if diff < 60 {
                    format!("{} {}", diff, words[i][0])
                } else if diff < 3600 {
                    format!("{} {}", diff / 60, words[i][1])
                } else if diff < 86400 {
                    format!("{} {}", diff / 3600, words[i][2])
                } else if diff < 7*86400 {
                    format!("{} {}", diff / 86400, words[i][3])
                } else if diff < 30*86400 {
                    format!("{} {}", diff / (7*86400), words[i][4])
                } else if diff < 365*86400 {
                    format!("{} {}", diff / (30*86400), words[i][5])
                } else {
                    format!("{} {}", diff / (365*86400), words[i][6])
                };
            }

            // Format group
            let school = user_info.user_groups.groups().get("school").map(|s| s.as_str()).unwrap_or_default();
            match school {
                "insa-rouen" => {
                    let department = user_info.user_groups.groups().get("insa-rouen:department").map(|s| s.as_str()).unwrap_or_default();
                    match department {
                        "STPI1" | "STPI2" => {
                            if let (Some(c), Some(g)) = (user_info.user_groups.groups().get("insa-rouen:stpi:class"), user_info.user_groups.groups().get("insa-rouen:stpi:tp-group")) {
                                formatted_group = format!("{department}, {} {c}{g}", t("en classe"));
                            }
                        }
                        "ITI3" => {
                            if let Some(g) = user_info.user_groups.groups().get("insa-rouen:iti:group") {
                                formatted_group = format!("{department}, {} {g}", t("en groupe"));
                            }
                        }
                        department => formatted_group = department.to_string(),
                    }
                }
                "" => (),
                _ => alert(format!("Unknown school {school}")),
            };
        }

        let app_link = ctx.props().app_link.clone();
        let app_link2 = ctx.props().app_link.clone();
        let app_link3 = ctx.props().app_link.clone();
        let app_link4 = ctx.props().app_link.clone();

        let user_info = ctx.props().user_info.as_ref();
        let has_password = user_info.as_ref().map(|user_info| user_info.has_password).unwrap_or(true);

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
        let language_message = match SETTINGS.lang() {
            Lang::French => "Pour afficher l'interface dans langue de Molière.",
            Lang::English => "To display the interface in Shakespeare's language.",
        };

        template_html!(
            "src/settings/settings.html",
            onclick_logout = {ctx.link().callback(move |_| Msg::LogOut)},
            onclick_confirm = {ctx.link().callback(move |_| Msg::Confirm)},
            onclick_delete = {ctx.link().callback(move |_| Msg::Delete)},
            onclick_cancel = {ctx.link().callback(move |_| Msg::Cancel)},
            onclick_change_password = {move |_| app_link.send_message(AppMsg::SetPage(Page::ChangePassword))},
            onclick_change_password2 = {move |_| app_link4.send_message(AppMsg::SetPage(Page::ChangePassword))},
            onclick_change_email = {move |_| app_link2.send_message(AppMsg::SetPage(Page::ChangeEmail))},
            onclick_change_group = {move |_| app_link3.send_message(AppMsg::SetPage(Page::ChangeGroup))},
            last_password_mod = last_password_mod_str,
            ...
        )
    }
}
