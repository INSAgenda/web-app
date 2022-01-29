use yew::prelude::*;
use crate::{glider_selector::GliderSelector, App};
use std::sync::atomic::{AtomicUsize, Ordering};

lazy_static::lazy_static!{
    pub static ref SETTINGS: SettingStore = SettingStore {
        building_naming: AtomicUsize::new(0),
    };
}

pub enum BuildingNaming {
    Short = 0,
    Long,
}

pub struct SettingStore {
    building_naming: AtomicUsize,
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
}

pub enum Msg {
    Confirm,
    BuildingNamingChange(usize),
}

#[derive(Properties, Clone)]
pub struct SettingsProps {
    pub app_link: yew::html::Scope<App>,
}

impl PartialEq for SettingsProps {
    fn eq(&self, _other: &Self) -> bool { true }
}

pub struct Settings {}

impl Component for Settings {
    type Message = Msg;
    type Properties = SettingsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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
                            values = { vec!["Automatique", "Sombre", "Clair"] }
                            selected = 0 />
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
                <div class="red-button" onclick={ctx.link().callback(move |_| Msg::Confirm)}>{"Valider"}</div>
            </main>
            <footer>
            </footer>
        </>}
    }
}
