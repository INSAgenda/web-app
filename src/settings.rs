use crate::prelude::*;

lazy_static::lazy_static!{
    pub static ref SETTINGS: SettingStore = {
        let local_storage = window().local_storage().unwrap().unwrap();
        let theme = match local_storage.get_item("setting-theme").unwrap() {
            Some(theme) if theme == "dark" => 0,
            Some(theme) if theme == "light" => 1,
            Some(theme) => {alert(format!("Invalid theme {theme}")); 0},
            None => 2,
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

#[derive(PartialEq, Clone, Copy)]
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
    BuildingNamingChange(usize),
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
    fn eq(&self, _other: &Self) -> bool { true }
}

pub struct SettingsPage {}

impl Component for SettingsPage {
    type Message = Msg;
    type Properties = SettingsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Confirm => {
                ctx.props().app_link.send_message(AppMsg::SetPage(Page::Agenda));
                false
            }
            Msg::BuildingNamingChange(v) => {
                SETTINGS.set_building_naming(v);
                true
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
                let document = window.document().unwrap();
                let html = document.first_element_child().unwrap();
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
                        Err(_e) => alert("Impossible de se déconnecter !"),
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
        let mut verified_msg = String::new();
        let mut email = String::from(t("[inconnue]"));
        let mut promotion = String::from(t("[inconnue]"));
        let mut class = String::from(t("[inconnue]"));
        let mut last_password_mod_str = String::from(t("[indisponible]"));
        if let Some(user_info) = ctx.props().user_info.as_ref() {
            if !user_info.email.1 {
                verified_msg = String::from(t(" Elle n'a pas encore été vérifiée."));
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
                    format!("{} {}", diff / 7*86400, words[i][4])
                } else if diff < 365*86400 {
                    format!("{} {}", diff / 30*86400, words[i][5])
                } else {
                    format!("{} {}", diff / 365*86400, words[i][6])
                };
            }
            promotion = user_info.group_desc.promotion.to_string();
            class = format!("{}{}", user_info.group_desc.class, user_info.group_desc.class_half);
        }

        let app_link = ctx.props().app_link.clone();
        let app_link2 = ctx.props().app_link.clone();
        html! {
            <>
            <header>
                <a id="header-logo"  href="/agenda">
                    <img height="35" width="35" src="/assets/logo/logo.svg" alt="Insagenda logo"/> 
                    <h1 id="header-name">{"INSAgenda"}</h1>
                </a>
                <div class="logout-button" onclick={ctx.link().callback(move |_| Msg::LogOut)}>
                    <span>{t("Se déconnecter")}</span>
                    <img src="agenda/images/log-out.svg"/>
                </div>
            </header>
            <main id="settings-main">
                <h2>{t("Paramètres")}</h2>
                <div id="settings-container">
                    <section>
                        <h3>{t("Général")}</h3>
                        <div class="settings-group">
                            <div class="setting">
                                <h4>{t("Changer de mot de passse")}</h4>
                                <p>{format!("{} {}.", t("Votre mot de passe a été changé il y a"), last_password_mod_str)}</p>
                                <div class="primary-button" onclick={move |_| app_link.send_message(AppMsg::SetPage(Page::ChangePassword))}>{t("Modifier")}</div>
                            </div>
                            <div class="setting">
                                <h4>{t("Changer de classe")}</h4>
                                <p>{format!("{} {} {} {}.", t("Vous êtes actuellement en"), promotion, t("dans le groupe"), class)}</p>
                                <div class="primary-button">{t("Modifier")}</div>
                            </div>
                            <div class="setting">
                                <h4>{t("Adresse mail")}</h4>
                                <p>{format!("{} {email}.{verified_msg}", t("Votre adresse actuelle est"))}</p>
                                <div class="primary-button" onclick={move |_| app_link2.send_message(AppMsg::SetPage(Page::ChangeEmail))}>{t("Changer / Vérifier")}</div>
                            </div>
                        </div>
                    </section>
                    <section>
                        <h3>{t("Affichage")}</h3>
                        <div class="settings-group">
                            <div class="setting">
                                <h4>{t("Thème")}</h4>
                                <p>{t("Par défault, le thème est celui renseigné par votre navigateur.")}</p>
                                <GliderSelector
                                    values = { vec![t("Sombre"), t("Clair"), t("Système")] }
                                    on_change = { ctx.link().callback(Msg::ThemeChange) }
                                    selected = { SETTINGS.theme() as usize } />
                            </div>
                            <div class="setting">
                                <h4>{t("Langue")}</h4>
                                <p>{
                                    match SETTINGS.lang() {
                                        Lang::French => "Pour afficher l'interface dans langue de Molière.",
                                        Lang::English => "To display the interface in Shakespeare's language.",
                                    }
                                }</p>
                                <GliderSelector
                                    values = { vec!["Français", "English"] }
                                    on_change = { ctx.link().callback(Msg::LanguageChange) }
                                    selected = { SETTINGS.lang() as usize } />
                                
                            </div>
                            <div class="setting">
                                <h4>{t("Nom des bâtiments")}</h4>
                                <p>{
                                    match SETTINGS.building_naming() {
                                        BuildingNaming::Short => "Ex: Ma",
                                        BuildingNaming::Long => "Ex: Magellan",
                                    }
                                }</p>
                                <GliderSelector
                                    values = { vec![t("Court"), t("Long")] }
                                    on_change = { ctx.link().callback(Msg::BuildingNamingChange) }
                                    selected = { SETTINGS.building_naming() as usize } />
                            </div>
                        </div>
                    </section>
                </div>
                <div class="secondary-button " onclick={ctx.props().app_link.callback(|_| AppMsg::SetPage(Page::Agenda))}>{t("Annuler")}</div>
                <div class="primary-button" onclick={ctx.link().callback(move |_| Msg::Confirm)}>{t("Valider")}</div>
            </main>
            </>
        }
    }
}
