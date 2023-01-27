use web_sys::HtmlSelectElement;

use crate::{prelude::*, redirect};

/// What data is being changed
enum Data {
    /// password, new_password, confirm_password
    NewPassword(NodeRef, NodeRef, NodeRef),
    /// password, email
    Email(NodeRef, NodeRef),
    Group(UserGroups),
}

impl Data {
    /// Title to be displayed on top of the page
    fn h2(&self) -> &'static str {
        match self {
            Data::NewPassword(_, _, _) => t("Changer de mot de passe"),
            Data::Email(_, _) => t("Changer d'email"),
            Data::Group(_) => t("Changer de groupe"),
        }
    }

    /// Title to be displayed on top of the form
    fn h3(&self) -> &'static str {
        match self {
            Data::NewPassword(_, _, _) => t("Nouveau mot de passe"),
            Data::Email(_, _) => t("Nouvelle adresse email"),
            Data::Group(_) => t("Nouveau groupe"),
        }
    }
}

/// Message for the component `ChangeDataPage`
pub enum Msg {
    Submit,
    GroupSelectChanged(web_sys::Event),
    SetMessage(Option<String>),
    SetLoading(bool),
}

/// Properties for the component `ChangeDataPage`
#[derive(Properties, Clone)]
pub struct ChangeDataProps {
    pub kind: String,
    pub groups: Rc<Vec<GroupDesc>>,
    pub app_link: Scope<App>,
    pub user_info: Rc<Option<UserInfo>>,
}
impl PartialEq for ChangeDataProps {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.groups == other.groups
            && self.user_info == other.user_info
    }
}

/// The `ChangeDataPage` component
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
                "group" => Data::Group(ctx.props().user_info.as_ref().as_ref().map(|ui| ui.user_groups.clone()).unwrap_or_else(|| UserGroups::new_with_groups(BTreeMap::new()))),
                _ => unreachable!(),
            },
            message: None,
            is_loading: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetMessage(message) => {
                let update_required = self.message != message;
                self.message = message;
                update_required
            },
            Msg::SetLoading(is_loading) => {
                self.is_loading = is_loading;
                true
            },
            Msg::GroupSelectChanged(event) => {
                let target = match event.target() {
                    Some(target) => target,
                    _ => return false,
                };
                let select = match target.dyn_into::<HtmlSelectElement>() {
                    Ok(select) => select,
                    Err(_) => return false,
                };
                let name = match select.get_attribute("name") {
                    Some(name) => name,
                    None => return false,
                };
                let user_groups = match &mut self.data {
                    Data::Group(user_groups) => user_groups,
                    _ => return false,
                };
                let value = select.selected_value();
                user_groups.insert(name, value);
                true
            }
            Msg::Submit => {
                let mut new_user_info = (*(ctx.props().user_info)).clone();
                let has_password = new_user_info.as_ref().map(|user_info| user_info.has_password).unwrap_or(true);

                let body = match &self.data {
                    Data::NewPassword(password, new_password, confirm_password) => {
              
                        let password = password.cast::<HtmlInputElement>().map(|input| input.value()).unwrap_or_default();
                        let input = new_password.cast::<HtmlInputElement>().unwrap();
                        let new_password = input.value();

                        let input = confirm_password.cast::<HtmlInputElement>().unwrap();
                        let confirm_password = input.value();

                        // Check if all inputs are filled
                        if (has_password && password.is_empty()) || new_password.is_empty() || confirm_password.is_empty() {
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

                        // Update user info
                        if let Some(new_user_info) = &mut new_user_info {
                            new_user_info.last_password_mod = Some(now_ts());
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

                        // Update user info
                        if let Some(new_user_info) = &mut new_user_info {
                            new_user_info.email.0 = email.clone();
                        }

                        format!(r#"{{
                            "password": "{}",
                            "new_email": "{}"
                        }}"#, password.replace('"', "\\\""), email.replace('"', "\\\""))
                    },
                    Data::Group(input_user_groups) => {
                        let mut input_user_groups = input_user_groups.clone();

                        // Make sure all fields are set
                        for group in ctx.props().groups.iter() {
                            let required = group.required_if.as_ref().map(|ri| input_user_groups.matches(ri)).unwrap_or(true);
                            if required && input_user_groups.groups().get(&group.id).is_none() {
                                ctx.link().send_message(Msg::SetMessage(Some(format!("{} ({})", t("Tous les champs doivent être remplis."), group.name.0.to_lowercase())))); // TODO: translation
                                return true;
                            }
                        }

                        // Sweep groups
                        input_user_groups.sweep(&ctx.props().groups);

                        // Update user info
                        if let Some(new_user_info) = &mut new_user_info {
                            new_user_info.user_groups = input_user_groups.to_owned();
                        }

                        format!(r#"{{
                            "new_group": "{}"
                        }}"#, input_user_groups.format_to_string())
                    },
                };

                ctx.link().send_message(Msg::SetLoading(true));

                let mut init = web_sys::RequestInit::new();
                init.body(Some(&JsValue::from_str(&body)));
                
                let app_link = ctx.props().app_link.clone();
                let link = ctx.link().clone();
                spawn_local(async move   {
                    match post_api_request("account", init, vec![("Content-Type", "application/json")]).await {
                        Ok(response) => {
                            let response: web_sys::Response = response.dyn_into().unwrap();
                            match response.status() {
                                200 => {
                                    app_link.send_message(AppMsg::SetPage(Page::Agenda));
                                    if let Some(new_user_info) = new_user_info {
                                        app_link.send_message(AppMsg::UserInfoSuccess(new_user_info));
                                    }
                                }
                                400 | 500 => {
                                    if let Ok(json) = response.json() {
                                        if let Ok(json) = JsFuture::from(json).await {
                                            let api_error = ApiError::from(json);
                                            api_error.handle_api_error();
                                            return;
                                        }
                                    }
                                    alert("Invalid json data returned after posting to /account");
                                }
                                _ => {
                                    alert(t("Une erreur inconnue est survenue. Veuillez contacter le support: support@insagenda.fr"));
                                }
                            }
                                
                        }
                        Err(_) => {
                            alert(t("Impossible de se connecter au serveur. Veuillez contacter le support: support@insagenda.fr"));
                        }
                    }
                    link.send_message(Msg::SetLoading(false));       
                });

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Build the custom part of the form
        let user_info = ctx.props().user_info.as_ref();
        let display_password = user_info.as_ref().map(|user_info| user_info.has_password).unwrap_or(true);
        let mut opt_msg = None;
        
        let inputs = match &self.data {
            Data::NewPassword(password, new_password, confirm_password) => html! {<>    
                if display_password {     
                    <div class="labeled-input">
                        <input type="password" placeholder="Password" id="password-input1" autocomplete="password" ref={password.clone()} />
                        <label for="password-input1">{t("Mot de passe actuel").to_string() + if user_info.is_some() {""} else {t(" (vide si inexistant)")}}</label>
                    </div>
                }
                <div class="labeled-input">
                    <input type="password" placeholder="New password" id="password-input2" autocomplete="new-password" ref={new_password.clone()}/>
                    <label for="password-input2">{t("Nouveau mot de passe")}</label>
                </div>
                <div class="labeled-input">
                    <input type="password" placeholder="Password (confirmation)" id="password-input3" autocomplete="new-password" ref={confirm_password.clone()} />
                    <label for="password-input3">{t("Nouveau mot de passe (confirmation)")}</label>
                </div>
            </>},
            Data::Email(password, email) if display_password => html! {<>
                <div class="labeled-input">
                    <input type="email" placeholder="Email" id="email" autocomplete="email" ref={email.clone()}/>
                    <label for="email">{t("Adresse email de l'INSA")}</label>
                </div>
                <div class="labeled-input">
                    <input type="password" placeholder="Password" id="password-input1" autocomplete="password" ref={password.clone()} />
                    <label for="password-input1">{t("Mot de passe actuel")}</label>
                </div>
                <br/>
                <p>{t("Un email de confirmation vous sera immédiatement envoyé.")}</p>
            </>},
            Data::Group(input_user_groups) => {
                if ctx.props().groups.is_empty() {
                    return html! {<>{t("Page indisponible, veuillez réessayer plus tard.")}</>};
                }

                // Display a message if the current groups are invalid
                if let Some(user_info) = user_info.as_ref() {
                    if user_info.user_groups.needs_correction(&ctx.props().groups) {
                        opt_msg = Some("Votre groupe actuel n'est plus valide, veuillez rentrer les informations manquantes.");
                    }
                }

                ctx.props().groups.iter().map(|group| {
                    let GroupDesc { id, name, help, values, required_if } = group;
                    let name = if SETTINGS.lang() == Lang::French { &name.0 } else { &name.1 };
                    let help = if SETTINGS.lang() == Lang::French { &help.0 } else { &help.1 };
                    let required = required_if.as_ref().map(|ri| input_user_groups.matches(ri)).unwrap_or(true);
                    let style = if required {"display: block;"} else {"display: none;"};
                    let missing = input_user_groups.groups().get(id).is_none();
                    let span_style = if missing { "display: none;" } else { "" };
                    let classes = if missing {"dropdown-list dropdown-list-missing"} else {"dropdown-list"};
                    html! {
                        <div class="dropdown-list-box" style={style}>
                            <span style={span_style}>{name}</span>
                            <select required=true class={classes} name={id.clone()} onchange={ctx.link().callback(Msg::GroupSelectChanged)}>
                                <option disabled=true selected={missing}>{name}</option>
                                {
                                    values
                                        .iter()
                                        .map(|(v, l)| html!{
                                            <option value={v.clone()} selected={input_user_groups.groups().get(id) == Some(v)}>
                                                {if SETTINGS.lang() == Lang::French { &l.0 } else { &l.1 }}
                                            </option>
                                        })
                                        .collect::<Html>()
                                }
                            </select>
                        </div>
                    }
                }).collect()
            },
            Data::Email(_password, _email) => {redirect("agenda"); html! {}}
        };

        let app_link = ctx.props().app_link.clone();
        let app_link2 = ctx.props().app_link.clone();
        let link = ctx.link().clone();
        template_html!(
            "src/change_data/change_data.html",
            onclick_settings = {move |_| app_link.send_message(AppMsg::SetPage(Page::Settings))},
            onclick_settings2 = {move |_| app_link2.send_message(AppMsg::SetPage(Page::Settings))},
            onclick_submit = {move |_| link.send_message(Msg::Submit)},
            is_loading = {self.is_loading},
            h2 = {self.data.h2()},
            h3 = {self.data.h3()},
            opt_error_message = {&self.message},
            ...
        )
    }
}
