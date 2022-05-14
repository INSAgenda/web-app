use crate::prelude::*;

pub enum Msg {
    SendConfirmationEmail,
    SendNewEmail,
    SetMessage(Option<String>),
    SetLoading(bool),
}

pub struct EditEmailPage {
    password: NodeRef,
    new_email: NodeRef,
    message: Option<String>,
    is_loading: bool,
}

impl Component for EditEmailPage {
    type Message = Msg;
    type Properties = SettingsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            password: NodeRef::default(),
            new_email: NodeRef::default(),
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
            Msg::SendNewEmail => {
                // Get inputs
                let input = self.password.cast::<HtmlInputElement>().unwrap();
                let password = input.value();


                let input = self.new_email.cast::<HtmlInputElement>().unwrap();
                let new_email = input.value();

                // Check if all inputs are filled
                if password.is_empty() || new_email.is_empty() {
                    ctx.link().send_message(Msg::SetMessage(Some(t("Tous les champs doivent être remplis.").to_string())));
                    return true;
                }

                ctx.link().send_message(Msg::SetLoading(true));


                let mut init = web_sys::RequestInit::new();
                init.body(Some(&JsValue::from_str(
                    &format!(r#"{{
                        "password": "{}",
                        "new_password": ""
                        }}"#, password.replace('"', "\\\""))),
                    ));
                
                let app_link = ctx.props().app_link.clone();
                let link = ctx.link().clone();
                spawn_local(async move   {
                    match post_api_request("account", init, vec![("Content-Type", "application/json")]).await{
                        Ok(response) => {
                            let response: web_sys::Response = response.dyn_into().unwrap();
                            match response.status() {
                                200 => {
                                    app_link.send_message(AppMsg::SetPage(Page::Agenda));
                                    link.send_message(Msg::SetMessage(None));
                                }
                                400 => {
                                    let json = JsFuture::from(response.json().unwrap()).await.map_err(|e| e).unwrap();
                                    let error: KnownApiError = json.into_serde().expect("JSON parsing issue");
                                    link.send_message(Msg::SetMessage(Some(error.message_fr)) );
                                }
                                _ => {
                                    alert(t("Une erreur inconnue est survenue. Veuillez contacter le support: support@insagenda.fr"));
                                }
                            }
                                
                        }
                        Err(_) => {
                            alert(t("Impossible de se connecter au le serveur. Veuillez contacter le support: support@insagenda.fr"));
                        }
                    }
                    link.send_message(Msg::SetLoading(false));       
                });
                true
            },
            Msg::SendConfirmationEmail => {
                true
            },
            
           
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let app_link = ctx.props().app_link.clone();
        html! {
            <>
            <header>
                <a id="header-logo" href="../index.html">
                    <img src="/assets/logo/logo.svg" alt="INSAgenda logo"/> 
                    <h1 id="header-name">{"INSAgenda"}</h1>
                </a>
                <button id="settings-button" onclick={move |_| app_link.send_message(AppMsg::SetPage(Page::Settings))}/>
            </header>
            <section class="section-page-title">
                <h2 class="page-title">{t("Changement d'email")}</h2>
                <div class="divider-bar"></div>
            </section>
            <main class="centred" id="auth">
                <h3 class="login-title">{t("Changer son mot de passse")}</h3>
                <form class="centred">
                    <div class="labeled-input">
                        <input type="password" placeholder="Password" id="password-input1" autocomplete="password" ref={self.password.clone()} />
                        <label for="password-input1">{t("Mot de passe actuel")}</label>
                    </div>
                
                    if self.is_loading{
                        <div class="lds-ring"><div></div><div></div><div></div><div></div></div>
                    }else{
                        if self.message.is_some() {
                            <span class="error-message">
                                {self.message.clone().unwrap()}
                            </span>
                        }
                        <input type="button" class="primary-button" id="submit-button" value={t("Confirmer")} onclick={ctx.link().callback(|_| Msg::SendNewEmail) }/>
                    }

                </form>   
            </main>
            </>
        }
    }
}
