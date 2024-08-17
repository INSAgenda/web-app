pub use crate::{
    agenda::*, alert::*, api::*, calendar::*, checkbox::*,
    colors::*, comment::*, event::*, friends::*, glider_selector::*, log, popup::Popup, popup::*,
    settings::*, tabbar::*, translation::*, util::*, app::*, pages::*,
};
pub use calendrier::{
    Date as RepublicanDate, DateTime as RepublicanDateTime, Month as RepublicanMonth,
};
pub use chrono::{
    DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc,
    Weekday,
};
pub use chrono_tz::{Europe::Paris, Tz};
pub use common::{Event as RawEvent, *};
pub use js_sys::{Array, Function, Reflect};
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
    ops::Deref,
    rc::Rc,
    time::Duration,
};
pub use wasm_bindgen::{prelude::*, JsCast, JsValue};
pub use wasm_bindgen_futures::{spawn_local, JsFuture};
pub use web_sys::{HtmlElement, HtmlInputElement, Request, RequestInit};
pub use yew::{html::Scope, prelude::*};
pub use yew_template::template_html;

pub type AppLink = yew::html::Scope<crate::App>;
pub type AgendaLink = yew::html::Scope<crate::agenda::Agenda>;
pub type PopupLink = yew::html::Scope<crate::popup::Popup>;
