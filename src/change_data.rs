use crate::prelude::*;

pub enum Msg {
    Submit,
    SetMessage(Option<String>),
    SetLoading(bool),
}

pub enum Data {
    NewPassword(NodeRef, NodeRef, NodeRef),
    Email(NodeRef, NodeRef),
}

#[derive(Properties, Clone)]
pub struct ChangeDataProps {
    pub app_link: Scope<App>,
    pub kind: String,
}

impl PartialEq for ChangeDataProps {
    fn eq(&self, _other: &Self) -> bool { true }
}

pub struct ChangeDataPage {
    data: Data,
    message: Option<String>,
    is_loading: bool,
}

impl Component for ChangeDataPage {
    type Message = Msg;
    type Properties = ChangeDataProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            data: match ctx.props().kind.as_str() {
                "new_password" => Data::NewPassword(NodeRef::default(), NodeRef::default(), NodeRef::default()),
                "email" => Data::Email(NodeRef::default(), NodeRef::default()),
                _ => unreachable!(),
            },
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
            Msg::Submit => {
                let body = match &self.data {
                    Data::NewPassword(password, new_password, confirm_password) => {
                        // Get inputs
                        let input = password.cast::<HtmlInputElement>().unwrap();
                        let password = input.value();

                        let input = new_password.cast::<HtmlInputElement>().unwrap();
                        let new_password = input.value();

                        let input = confirm_password.cast::<HtmlInputElement>().unwrap();
                        let confirm_password = input.value();

                        // Check if all inputs are filled
                        if password.is_empty() || new_password.is_empty() || confirm_password.is_empty() {
                            ctx.link().send_message(Msg::SetMessage(Some(t("Tous les champs doivent être remplis.").to_string())));
                            return true;
                        }

                        // Check if passwords match
                        if new_password != confirm_password {
                            ctx.link().send_message(Msg::SetMessage(Some(t("Les mots de passe ne correspondent pas.").to_string())));
                            return true;
                        }

                        // Check if new password is same as old password
                        if new_password == password {
                            ctx.link().send_message(Msg::SetMessage(Some(t("Le nouveau mot de passe doit être différent du mot de passe actuel.").to_string())));
                            return true;
                        }

                        format!(r#"{{
                            "password": "{}",
                            "new_password": "{}"
                        }}"#, password.replace('"', "\\\""), new_password.replace('"', "\\\""))
                    },
                    Data::Email(password, email) => {
                        // Get inputs
                        let input = password.cast::<HtmlInputElement>().unwrap();
                        let password = input.value();

                        let input = email.cast::<HtmlInputElement>().unwrap();
                        let email = input.value();

                        // Check if all inputs are filled
                        if password.is_empty() || email.is_empty() {
                            ctx.link().send_message(Msg::SetMessage(Some(t("Tous les champs doivent être remplis.").to_string())));
                            return true;
                        }

                        format!(r#"{{
                            "password": "{}",
                            "email": "{}"
                        }}"#, password.replace('"', "\\\""), email.replace('"', "\\\""))
                    },
                };

                ctx.link().send_message(Msg::SetLoading(true));

                let mut init = web_sys::RequestInit::new();
                init.body(Some(&JsValue::from_str(&body)));
                
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
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let inputs = match &self.data {
            Data::NewPassword(password, new_password, confirm_password) => html! {<>
                <div class="labeled-input">
                    <input type="password" placeholder="Password" id="password-input1" autocomplete="password" ref={password.clone()} />
                    <label for="password-input1">{t("Mot de passe actuel")}</label>
                </div>
                <div class="labeled-input">
                    <input type="password" placeholder="New password" id="password-input2" autocomplete="new-password" ref={new_password.clone()}/>
                    <label for="password-input2">{t("Nouveau mot de passe")}</label>
                </div>
                <div class="labeled-input">
                    <input type="password" placeholder="Password (confirmation)" id="password-input3" autocomplete="new-password" ref={confirm_password.clone()} />
                    <label for="password-input3">{t("Nouveau mot de passe (confirmation)")}</label>
                </div>
            </>},
            Data::Email(password, email) => html! {<>
                <div class="labeled-input">
                    <input type="password" placeholder="Password" id="password-input1" autocomplete="password" ref={password.clone()} />
                    <label for="password-input1">{t("Mot de passe actuel")}</label>
                </div>
                <div class="labeled-input">
                    <input type="email" placeholder="Email" id="email" autocomplete="email" ref={email.clone()}/>
                    <label for="email">{t("Adresse email de l'INSA")}</label>
                </div>
            </>},
        };
        
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
                <h2 class="page-title">{t("Changement de mot de passe")}</h2>
                <div class="divider-bar"></div>
            </section>
            <main class="centred" id="auth">
                <h3 class="login-title">{t("Changer son mot de passse")}</h3>
                <form class="centred">
                    {inputs}
                    if self.is_loading{
                        <div class="lds-ring"><div></div><div></div><div></div><div></div></div>
                    }else{
                        if self.message.is_some() {
                            <span class="error-message">
                                {self.message.clone().unwrap()}
                            </span>
                        }
                        <input type="button" class="primary-button" id="submit-button" value={t("Confirmer")} onclick={ctx.link().callback(|_| Msg::Submit) }/>
                    }

                </form>   
            </main>
            </>
        }
    }
}
