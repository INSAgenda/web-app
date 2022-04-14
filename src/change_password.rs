use yew::prelude::*;
use crate::{event::EventComp, App};


impl App{
    pub fn view_change_password(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            <header>
                <a id="header-logo" href="../index.html">
                    <img src="/assets/elements/webLogo.svg" alt="INSAgenda logo"/> 
                    <h1 id="header-name">{"INSAgenda"}</h1>
                </a>
                <button id="settings-button" onclick={ctx.link().callback(|_| crate::Msg::SetPage(crate::Page::Settings))}/>
            </header>
            <section class="section-page-title">
                <h2 class="page-title">{"Changer le mot de passe"}</h2>
                <div class="divider-bar"></div>
            </section>
            <main class="centred" id="auth">
                <form class="centred">
                    <div class="labeled-input">
                        <input type="password" placeholder="Password" id="password-input1" autocomplete="password" />
                        <label for="password-input1">{"Mot de passe actuel"}</label>
                        <span class="error-message-text">{"Veuillez entrer votre mot de passe actuel."}</span>
                    </div>
                    <div class="labeled-input">
                        <input type="password" placeholder="New password" id="password-input2" autocomplete="new-password" />
                        <label for="password-input2">{"Nouuveau mot de passe"}</label>
                        <span class="error-message-text">{"Mot de passe invalide."}</span>
                    </div>
                    <div class="labeled-input">
                        <input type="password" placeholder="Password (confirmation)" id="password-input3" autocomplete="new-password" />
                        <label for="password-input3">{"Mot de passe (confirmation)"}</label>
                        <span class="error-message-text">{"Les deux mots de passe sont différents."}</span>
                    </div>
                    <input type="button" class="red-button form-button" id="submit-button" value="Confirmer"/>
                </form>   
            </main>
            </>
        }
    }
        
    
}