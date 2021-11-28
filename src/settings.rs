use yew::prelude::*;
use crate::glider_selector::GliderSelector;

pub enum Msg {
    AddOne,
}

pub struct Settings {
    link: ComponentLink<Self>,
    value: i64,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            value: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
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
                        <div class="big-white-button small-button">{"Modifier"}</div>
                    </div>
                    <br/>
                    <div class="setting">
                        <h3>{"Adresse mail"}</h3>
                        <p>{"Votre adresse actuelle est foobar@insa-rouen.fr."}</p>
                        <div class="big-white-button small-button">{"Modifier"}</div>
                    </div>
                    <br/>
                    <div class="setting">
                        <h3>{"Changer le type d'authentification"}</h3>
                        <p>{"L'authentification par email consiste a rentrer un code unique qui vous sera envoyé par email."}</p>
                        <GliderSelector values=vec!["Email", "Mot de passe", "Email + Mot de passe"] />
                    </div>
                </div>
                <h2>{"Affichage"}</h2>
                <div class="settings-group">
                    {"Enregistrer"}
                </div>
                <div class="big-red-button small-button">{"Valider"}</div>
            </main>
            <footer>
            </footer>
        </>}
    }
}
