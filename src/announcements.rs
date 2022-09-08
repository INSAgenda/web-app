use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct AnnouncementImpressions {
    /// Number of times the announcement was displayed
    pub count: u64,
    /// Whether the user has dismissed the announcement
    pub closed: bool,
}

fn load_impressions() -> Option<HashMap<String, AnnouncementImpressions>> {
    let local_storage = window().local_storage().unwrap().unwrap();
    let impression_counts_str = local_storage.get("announcement_impressions").ok().flatten()?;
    serde_json::from_str(&impression_counts_str).ok()
}

fn save_impressions(impression_counts: &HashMap<String, AnnouncementImpressions>) {
    let local_storage = window().local_storage().unwrap().unwrap();

    let impression_counts_str = match serde_json::to_string(impression_counts) {
        Ok(impression_counts_str) => impression_counts_str,
        _ => return,
    };

    let _ = local_storage.set("announcement_impressions", &impression_counts_str);
}

/// Close the currently displayed announcement
/// 
/// Function to be called by App in its update method
pub fn update_close_announcement(app: &mut App) -> bool {
    // Make announcement not displayed
    let announcement = match app.displayed_announcement.take() {
        Some(announcement) => announcement,
        None => return true,
    };

    // Remember user's choice for future runs
    let mut impressions = load_impressions().unwrap_or_default();
    let mut impression_data = impressions.get(&announcement.id).cloned().unwrap_or_default();
    impression_data.closed = true;
    impressions.insert(announcement.id, impression_data);
    save_impressions(&impressions);

    // Try to hide the announcement without refreshing the display
    // TODO: enable this on desktop only
    /*let announcement_el = match window().doc().get_element_by_id("announcement") {
        Some(announcement_el) => announcement_el,
        None => return true,
    };
    let announcement_el: HtmlElement = announcement_el.dyn_into().unwrap();
    announcement_el.style().set_property("display", "none").unwrap();
    return false;*/

    true
}

/// Select the announcement to be displayed from a list of announcements
pub fn select_announcement(announcements: &[AnnouncementDesc]) -> Option<AnnouncementDesc> {
    let mut impressions = load_impressions().unwrap_or_default();
    let now = (js_sys::Date::new_0().get_time() / 1000.0) as u64;
    
    for a in announcements {
        if (a.start_ts..=a.end_ts).contains(&now) {
            let mut impression_data = impressions.get(&a.id).cloned().unwrap_or_default();
            if impression_data.closed {
                continue;
            }
            if let Some(max_impressions) = a.max_impressions {
                if impression_data.count >= max_impressions {
                    continue
                }
            }
            impression_data.count += 1;
            impressions.insert(a.id.clone(), impression_data);
            save_impressions(&impressions);
            return Some(a.clone());
        }
    }

    None
}

/// Generate Yew HTML for displaying an announcement
pub fn view_announcement(a: &AnnouncementDesc, ctx: &Context<App>) -> Html {
    let mut classes = String::new();
    if a.ty == ContentType::Text {
        classes.push_str(" text-announcement");
    }

    // Create script element if required
    let script = a.script.as_ref().map(|s| {
        let script = window()
            .doc()
            .create_element("script")
            .unwrap();
        script.clone().dyn_into::<HtmlElement>().unwrap().set_inner_text(s);
        let node = web_sys::Node::from(script);
        yew::virtual_dom::VNode::VRef(node)
    });

    // Create close button
    let close_button = match a.closable {
        true => html! {
            <svg id="close-announcement-button" xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x" viewBox="0 0 16 16" onclick={ctx.link().callback(|_| AppMsg::CloseAnnouncement)}>
                <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"/>
            </svg>
        },
        false => html! {},
    };

    // Create main structure
    html! {
        <div id="announcement" class={classes}>
            {close_button}
            {match a.ty {
                ContentType::Text => html! {<>
                    <div>{a.title.as_str()}</div>
                    <p>
                        {match SETTINGS.lang() {
                            Lang::French => a.content_fr.as_deref().unwrap_or_default(),
                            Lang::English => a.content_en.as_deref().unwrap_or_default(),
                        }}
                    </p>
                </>},
                ContentType::Html => {
                    let div = window()
                        .doc()
                        .create_element("div")
                        .unwrap();
                    match SETTINGS.lang() {
                        Lang::French => div.set_inner_html(a.content_fr.as_deref().unwrap_or_default()),
                        Lang::English => div.set_inner_html(a.content_en.as_deref().unwrap_or_default()),
                    }
                    let node = web_sys::Node::from(div);
                    yew::virtual_dom::VNode::VRef(node)
                }
            }}
            if let Some(script) = script {
                {script}
            }
        </div>
    }
}
