use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlInputElement, Response};
use yew::prelude::*;
use crate::{App, api::{post_api_request, KnownApiError}, alert::alert};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::glider_selector::GliderSelector;

#[derive(Clone, PartialEq, Default)]
struct NewPasswordForm {
    pub password: NodeRef,
    pub new_password: NodeRef,
    pub confirm_password: NodeRef,
}

lazy_static::lazy_static!{
    pub static ref SETTINGS: SettingStore = {
        let window = web_sys::window().unwrap();
        let local_storage = window.local_storage().unwrap().unwrap();
        let theme = match local_storage.get_item("setting-theme").unwrap() {
            Some(theme) if theme == "dark" => 0,
            _ => 1,
        };

        SettingStore {
            building_naming: AtomicUsize::new(0),
            theme: AtomicUsize::new(theme),
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
}

pub struct SettingStore {
    building_naming: AtomicUsize,
    theme: AtomicUsize,
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
    }

    pub fn theme(&self) -> Theme {
        match self.theme.load(Ordering::Relaxed) {
            0 => Theme::Dark,
            1 => Theme::Light,
            _ => unreachable!(),
        }
    }

    fn set_theme(&self, theme: usize) {
        self.theme.store(theme, Ordering::Relaxed);
    }
}

pub enum Msg {
    Confirm,
    BuildingNamingChange(usize),
    ThemeChange(usize),
    SetPage(crate::settings::Page),
    SendNewPassword,
    SetMessage(Option<String>),
    SetLoading(bool),
}

#[derive(Properties, Clone)]
pub struct SettingsProps {
    pub app_link: yew::html::Scope<App>,
}

impl PartialEq for SettingsProps {
    fn eq(&self, _other: &Self) -> bool { true }
}

#[derive(PartialEq)]
pub enum Page{
    Main,
    ChangePassword
}
pub struct Settings {
    page: Page,
    new_password_form: NewPasswordForm,
    message: Option<String>,
    is_loading: bool,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = SettingsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            page: Page::Main,
            new_password_form: NewPasswordForm::default(),
            message: None,
            is_loading: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Confirm => {
                ctx.props().app_link.send_message(crate::Msg::SetPage(crate::Page::Agenda));
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
                    _ => unreachable!(),
                };

                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let html = document.first_element_child().unwrap();
                html.set_attribute("data-theme", theme).unwrap();

                let storage = window.local_storage().unwrap().unwrap();
                storage.set_item("setting-theme", theme).unwrap();

                true
            }
            Msg::SetPage(page) => {
                let history = web_sys::window().unwrap().history().unwrap();     
                match self.page {
                    Page::Main => {
                        history.push_state_with_url(&JsValue::from_str(""), "Settings", Some("")).unwrap();
                    }
                    Page::ChangePassword => {
                        history.push_state_with_url(&JsValue::from_str("change-password"), "Nouveau mot de passe", Some("change-password")).unwrap();
                    }
                }
                    
                self.page = page;
                true
            },
            Msg::SetMessage(message) => {
                self.message = message;
                true
            },
            Msg::SetLoading(is_loading) => {
                self.is_loading = is_loading;
                true
            },
            Msg::SendNewPassword => {
                if self.page != Page::ChangePassword {
                    return true; // if this message was sent, rendering may not have been applied
                }

                // Get inputs
                let input = self.new_password_form.password.cast::<HtmlInputElement>().unwrap();
                let password = input.value();

                let input = self.new_password_form.new_password.cast::<HtmlInputElement>().unwrap();
                let new_password = input.value();

                let input = self.new_password_form.confirm_password.cast::<HtmlInputElement>().unwrap();
                let confirm_password = input.value();

                // Check if all inputs are filled
                if password.is_empty() || new_password.is_empty() || confirm_password.is_empty() {
                    ctx.link().send_message(Msg::SetMessage(Some("Tous les champs doivent être remplis.".to_string())));
                    return true;
                }

                // Check if passwords match
                if new_password != confirm_password {
                    ctx.link().send_message(Msg::SetMessage(Some("Les mots de passe ne correspondent pas.".to_string())));
                    return true;
                }
                
                // Check if new password is same as old password
                if new_password == password {
                    ctx.link().send_message(Msg::SetMessage(Some("Le nouveau mot de passe doit être différent du mot de passe actuel.".to_string())));
                    return true;
                }

                ctx.link().send_message(Msg::SetLoading(true));


                let mut init = web_sys::RequestInit::new();
                init.body(Some(&JsValue::from_str(
                    &format!(r#"{{
                        "password": "{}",
                        "new_password": "{}"
                        }}"#, password, new_password)),
                    ));
                
                let app_link = ctx.props().app_link.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move   {
                    match post_api_request("account", init, vec![("Content-Type", "application/json")]).await{
                        Ok(response) => {
                            let response: Response = response.dyn_into().unwrap();
                            match response.status() {
                                200 => {
                                    app_link.send_message(crate::Msg::SetPage(crate::Page::Agenda));
                                    link.send_message(Msg::SetMessage(None));
                                },
                                400 => {
                                    let json = JsFuture::from(response.json().unwrap()).await.map_err(|e| e).unwrap();
                                    let error: KnownApiError = json.into_serde().expect("JSON parsing issue");
                                    link.send_message(Msg::SetMessage(Some(error.message_fr)) );
                                }
                                500..=599 => {
                                    alert("Une erreur interne est survenue. Veuillez contacter le support: support@insagenda.fr");
                                }
                                _ => {
                                    alert("Une erreur inconnue est survenue. Veuillez contacter le support: support@insagenda.fr");
                                }
                            }
                                
                        }
                        Err(_) => {
                            alert("Impossible de contacter le serveur, une erreur est survenue. Veuillez contacter le support: support@insagenda.fr")
                        }
                    }
                    link.send_message(Msg::SetLoading(false));       
                });
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.page {
            Page::Main => self.view_main(ctx),
            Page::ChangePassword => self.view_change_password(ctx),
        }
    }
}

impl Settings{

    fn view_main(&self, ctx: &Context<Self>) -> Html{
        html! {
            <>
            <header class="pseudo-page-header">
                <button class="back-button" onclick={ctx.props().app_link.callback(|_| crate::Msg::SetPage(crate::Page::Agenda))} />
                <h1>{"Paramètres"}</h1>
            </header>
            <main id="settings-main">
                <h2>{"Paramètres du compte"}</h2>
                <div class="settings-group">
                    <div class="setting">
                        <h3>{"Mot de passe"}</h3>
                        <p>{"Votre mot de passe a été changé pour la dernière fois le 12/11/2021 à 12:49."}</p>
                        <div class="white-button small-button" onclick={ctx.link().callback(|_| Msg::SetPage(crate::settings::Page::ChangePassword))}>{"Modifier"}</div>
                    </div>
                    <div class="setting">
                        <h3>{"Adresse mail"}</h3>
                        <p>{"Votre adresse actuelle est foobar@insa-rouen.fr."}</p>
                        <div class="white-button small-button">{"Modifier"}</div>
                    </div>
                    <div class="setting">
                        <h3>{"Changer le type d'authentification"}</h3>
                        <GliderSelector
                            values = { vec!["Email", "Mot de passe", "Email + Mot de passe"] }
                            selected = 0 />
                        <p>{"L'authentification par email consiste a rentrer un code unique qui vous sera envoyé par email."}</p>
                    </div>
                </div>
                <h2>{"Affichage"}</h2>
                <div class="settings-group">
                    <div class="setting">
                        <h3>{"Thème"}</h3>
                        <GliderSelector
                            values = { vec!["Sombre", "Clair"] }
                            on_change = { ctx.link().callback(Msg::ThemeChange) }
                            selected = { SETTINGS.theme() as usize } />
                        <p>{"Par défault, le thème est celui renseigné par votre navigateur."}</p>
                    </div>
                    <div class="setting">
                        <h3>{"Nom des bâtiments"}</h3>
                        <GliderSelector
                            values = { vec!["Court", "Long"] }
                            on_change = { ctx.link().callback(Msg::BuildingNamingChange) }
                            selected = { SETTINGS.building_naming() as usize } />
                        <p>{
                            match SETTINGS.building_naming() {
                                BuildingNaming::Short => "Ex: Ma",
                                BuildingNaming::Long => "Ex: Magellan",
                            }
                        }</p>
                    </div>
                </div>
                <div class="red-button form-button" onclick={ctx.link().callback(move |_| Msg::Confirm)}>{"Valider"}</div>
            </main>
            <footer>
            </footer>
            </>
        }
    }

    fn view_change_password(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            <header>
                <a id="header-logo" href="../index.html">
                    <img src="/assets/elements/webLogo.svg" alt="INSAgenda logo"/> 
                    <h1 id="header-name">{"INSAgenda"}</h1>
                </a>
                <button id="settings-button" onclick={ctx.link().callback(move |_| Msg::SetPage(Page::Main))}/>
            </header>
            <section class="section-page-title">
                <h2 class="page-title">{"Changer le mot de passe"}</h2>
                <div class="divider-bar"></div>
            </section>
            <main class="centred" id="auth">
                <h3 class="login-title">{"Changer son mot de passse"}</h3>
                <form class="centred">
                    <div class="labeled-input">
                        <input type="password" placeholder="Password" id="password-input1" autocomplete="password" ref={self.new_password_form.password.clone()} />
                        <label for="password-input1">{"Mot de passe actuel"}</label>
                    </div>
                    <div class="labeled-input">
                        <input type="password" placeholder="New password" id="password-input2" autocomplete="new-password" ref={self.new_password_form.new_password.clone()}/>
                        <label for="password-input2">{"Nouuveau mot de passe"}</label>
                    </div>
                    <div class="labeled-input">
                        <input type="password" placeholder="Password (confirmation)" id="password-input3" autocomplete="new-password" ref={self.new_password_form.confirm_password.clone()} />
                        <label for="password-input3">{"Mot de passe (confirmation)"}</label>
                    </div>
                    if self.is_loading{
                        <div class="lds-ring"><div></div><div></div><div></div><div></div></div>
                    }else{
                        if self.message.is_some() {
                            <span class="error-message">
                                {self.message.clone().unwrap()}
                            </span>
                        }
                        <input type="button" class="red-button form-button" id="submit-button" value="Confirmer" onclick={ctx.link().callback(|_| Msg::SendNewPassword) }/>
                    }

                </form>   
            </main>
            </>
        }
    }
}