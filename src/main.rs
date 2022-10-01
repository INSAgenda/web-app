mod alert;
mod announcements;
mod event;
mod settings;
mod agenda;
mod glider_selector;
mod util;
mod calendar;
mod slider;
mod api;
mod crash_handler;
mod colors;
mod change_data;
mod prelude;
mod translation;
mod popup;

use agenda::refresh_events;

use crate::{prelude::*, settings::SettingsPage, change_data::ChangeDataPage};

pub enum Page {
    Settings,
    ChangePassword,
    ChangeEmail,
    ChangeGroup,
    Agenda,
}

pub enum Msg {
    UserInfoSuccess(UserInfo),
    UserInfoFailure(ApiError),
    SetPage(Page),
    SilentSetPage(Page),
}


pub struct App {
    page: Page,
    user_info: Rc<Option<UserInfo>>,
}


impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        crash_handler::init();

        let now = chrono::Local::now();
        let now = now.with_timezone(&Paris);

        let link2 = ctx.link().clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::PopStateEvent| {
            let state = e.state().as_string();
            match state.as_deref() {
                Some("settings") => link2.send_message(Msg::SilentSetPage(Page::Settings)),
                Some("agenda") => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                Some("change-password") => link2.send_message(Msg::SilentSetPage(Page::ChangePassword)),
                Some("change-email") => link2.send_message(Msg::SilentSetPage(Page::ChangeEmail)),
                Some("change-group") => link2.send_message(Msg::SilentSetPage(Page::ChangeGroup)),
                _ if e.state().is_null() => link2.send_message(Msg::SilentSetPage(Page::Agenda)),
                _ => alert(format!("Unknown pop state: {:?}", e.state())),
            }
        }) as Box<dyn FnMut(_)>);
        window().add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();

        // Update user info
        let mut skip_user_info_loading = false;
        let mut user_info = None;
        if let Some((last_updated, cached_user_info)) = api::load_cached_user_info() {
            if last_updated > now.timestamp() - 60*5 {
                skip_user_info_loading = true;
            }
            user_info = Some(cached_user_info);
        }
        if !skip_user_info_loading {
            let link2 = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::load_user_info().await {
                    Ok(events) => link2.send_message(Msg::UserInfoSuccess(events)),
                    Err(e) => link2.send_message(Msg::UserInfoFailure(e)),
                }
            });
        }

        // Detect page
        let page = match window().location().hash() {
            Ok(hash) if hash == "#settings" => Page::Settings,
            Ok(hash) if hash == "#change-password" => Page::ChangePassword,
            Ok(hash) if hash == "#change-email" => Page::ChangeEmail,
            Ok(hash) if hash == "#change-group" => Page::ChangeGroup,
            Ok(hash) if hash.is_empty() => Page::Agenda,
            Ok(hash) => {
                alert(format!("Page {hash} not found"));
                Page::Agenda
            },
            _ => Page::Agenda,
        };

        Self {
            page,
            user_info: Rc::new(user_info),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UserInfoSuccess(user_info) => {
                let mut should_refresh = false;

                // If user's group changed, update the events
                if let Some(old_user_info) = self.user_info.as_ref() {
                    if old_user_info.group_desc != user_info.group_desc {
                        should_refresh = true;
                    }
                }

                // Update user info
                api::save_user_info_cache(&user_info);
                self.user_info = Rc::new(Some(user_info));

                should_refresh
            },
            Msg::UserInfoFailure(api_error) => {
                api_error.handle_api_error();
                false
            },
            Msg::SetPage(page) => {
                let history = window().history().expect("Failed to access history");                
                match &page {
                    Page::Settings => history.push_state_with_url(&JsValue::from_str("settings"), "Settings", Some("#settings")).unwrap(),
                    Page::Agenda => history.push_state_with_url(&JsValue::from_str("agenda"), "Agenda", Some("/agenda")).unwrap(),
                    Page::ChangePassword => history.push_state_with_url(&JsValue::from_str("change-password"), "Change password", Some("#change-password")).unwrap(),
                    Page::ChangeEmail => history.push_state_with_url(&JsValue::from_str("change-email"), "Change email", Some("#change-email")).unwrap(),
                    Page::ChangeGroup => history.push_state_with_url(&JsValue::from_str("change-group"), "Change group", Some("#change-group")).unwrap(),
                }
                self.page = page;
                true
            },
            Msg::SilentSetPage(page) => {
                self.page = page;
                true
            },
        }
    }
    
    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        crate::colors::COLORS_CHANGED.store(false, std::sync::atomic::Ordering::Relaxed);
    }
    
    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.page {
            Page::Agenda => html!(<Agenda app_link={ ctx.link().clone()} />),
            Page::Settings => html!( <SettingsPage app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
            Page::ChangePassword => html!( <ChangeDataPage kind="new_password" app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
            Page::ChangeEmail => html!( <ChangeDataPage kind="email" app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
            Page::ChangeGroup => html!( <ChangeDataPage kind="group" app_link={ ctx.link().clone() } user_info={Rc::clone(&self.user_info)} /> ),
        }
    }
}

/// Redirect the user
fn redirect(page: &str){
    let _ = window().location().set_href(page);
}

/// Prevent webdrivers from accessing the page
fn stop_bots(window: &web_sys::Window) {
    if js_sys::Reflect::get(&window.navigator(), &"webdriver".to_string().into()).unwrap().as_bool().unwrap_or(false) {
        panic!("Your browser failed load this page");
    }
}

/// Install service worker for offline access
fn install_sw(window: &web_sys::Window) {
    let future = JsFuture::from(window.navigator().service_worker().register("/sw.js"));
    spawn_local(async move {
        match future.await {
            Ok(_) => log!("Service worker doing well"),
            Err(e) => alert(format!("Failed to register service worker: {:?}", e)),
        }
    })
}

fn main() {
    let window = web_sys::window().expect("Please run the program in a browser context");
    stop_bots(&window);
    install_sw(&window);
    let doc = window.doc();
    let element = doc.get_element_by_id("render").unwrap();
    yew::start_app_in_element::<App>(element);
}
