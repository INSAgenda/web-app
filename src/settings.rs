use yew::prelude::*;
use crate::{glider_selector::GliderSelector, App};
use std::rc::Rc;

pub enum Msg {
    Confirm,
}

#[derive(Properties, Clone)]
pub struct SettingsProp {
    pub app_link: Rc<ComponentLink<App>>,
}

pub struct Settings {
    link: ComponentLink<Self>,
    app_link: Rc<ComponentLink<App>>,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = SettingsProp;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            app_link: props.app_link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Confirm => {
                self.app_link.send_message(crate::Msg::SetPage(crate::Page::Agenda));
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.app_link = props.app_link;
        true
    }

    fn view(&self) -> Html {
        html! {<>
            <header>
                <h1>{"Paramètres"}</h1>
            </header>
            <main id="settings-main">
                <h2>{"Paramètres du compte"}</h2>
                <div class="settings-group">
                    <div class="setting">
                        <h3>{"Mot de passe"}</h3>
                        <p>{"Votre mot de passe a été changé pour la dernière fois le 12/11/2021 à 12:49."}</p>
                        <div class="white-button small-button">{"Modifier"}</div>
                    </div>
                    <br/>
                    <br/>
                    <div class="setting">
                        <h3>{"Adresse mail"}</h3>
                        <p>{"Votre adresse actuelle est foobar@insa-rouen.fr."}</p>
                        <div class="white-button small-button">{"Modifier"}</div>
                    </div>
                    <br/>
                    <br/>
                    <div class="setting">
                        <h3>{"Changer le type d'authentification"}</h3>
                        <p>{"L'authentification par email consiste a rentrer un code unique qui vous sera envoyé par email."}</p>
                        <GliderSelector values=vec!["Email", "Mot de passe", "Email + Mot de passe"] />
                    </div>
                </div>
                <h2>{"Affichage"}</h2>
                <div class="settings-group">
                    <div class="setting">
                        <h3>{"Thème"}</h3>
                        <p>{"Par défault, le thème est celui renseigné par votre navigateur."}</p>
                        <GliderSelector values=vec!["Automatique", "Sombre", "Clair"] />
                    </div>
                </div>
                <div class="red-button" onclick=self.link.callback(move |_| Msg::Confirm)>{"Valider"}</div>
            </main>
            <footer>
            </footer>
        </>}
    }
}
