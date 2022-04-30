use yew::prelude::*;
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlInputElement, Response};
use crate::{api::{post_api_request, KnownApiError}, alert::alert};

pub enum Msg {
    SendNewPassword,
    SetMessage(Option<String>),
    SetLoading(bool),
}

pub struct ChangePasswordPage {
    password: NodeRef,
    new_password: NodeRef,
    confirm_password: NodeRef,
    message: Option<String>,
    is_loading: bool,
}

impl Component for ChangePasswordPage {
    type Message = Msg;

    type Properties = crate::settings::SettingsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            password: NodeRef::default(),
            new_password: NodeRef::default(),
            confirm_password: NodeRef::default(),
            message: None,
            is_loading: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetMessage(message) => {
                self.message = message;
                true
            },
            Msg::SetLoading(is_loading) => {
                self.is_loading = is_loading;
                true
            },
            Msg::SendNewPassword => {
                // Get inputs
                let input = self.password.cast::<HtmlInputElement>().unwrap();
                let password = input.value();

                let input = self.new_password.cast::<HtmlInputElement>().unwrap();
                let new_password = input.value();

                let input = self.confirm_password.cast::<HtmlInputElement>().unwrap();
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
                        }}"#, password.replace('"', "\\\""), new_password.replace('"', "\\\""))),
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
        let app_link = ctx.props().app_link.clone();
        html! {
            <>
            <header>
                <a id="header-logo" href="../index.html">
                    <img src="/assets/elements/webLogo.svg" alt="INSAgenda logo"/> 
                    <h1 id="header-name">{"INSAgenda"}</h1>
                </a>
                <button id="settings-button" onclick={move |_| app_link.send_message(crate::Msg::SetPage(crate::Page::Settings))}/>
            </header>
            <section class="section-page-title">
                <h2 class="page-title">{"Changer le mot de passe"}</h2>
                <div class="divider-bar"></div>
            </section>
            <main class="centred" id="auth">
                <h3 class="login-title">{"Changer son mot de passse"}</h3>
                <form class="centred">
                    <div class="labeled-input">
                        <input type="password" placeholder="Password" id="password-input1" autocomplete="password" ref={self.password.clone()} />
                        <label for="password-input1">{"Mot de passe actuel"}</label>
                    </div>
                    <div class="labeled-input">
                        <input type="password" placeholder="New password" id="password-input2" autocomplete="new-password" ref={self.new_password.clone()}/>
                        <label for="password-input2">{"Nouuveau mot de passe"}</label>
                    </div>
                    <div class="labeled-input">
                        <input type="password" placeholder="Password (confirmation)" id="password-input3" autocomplete="new-password" ref={self.confirm_password.clone()} />
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
