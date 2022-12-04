use crate::prelude::*;

#[derive(Properties, PartialEq, Eq)]
pub struct SortableProps {
    pub items: Vec<String>,
}

#[derive(Debug)]
struct Positions {
    width_max: usize,
    y_min: usize,
    rects: Vec<web_sys::DomRect>,
}

pub struct Sortable {
    id: usize,
    positions: Option<Positions>,
    ordered: Vec<usize>,
    on_mouse_down: wasm_bindgen::prelude::Closure<dyn std::ops::FnMut(web_sys::MouseEvent)>,
    on_mouse_move: wasm_bindgen::prelude::Closure<dyn std::ops::FnMut(web_sys::Event)>,
    on_mouse_up: wasm_bindgen::prelude::Closure<dyn std::ops::FnMut(web_sys::Event)>,
}

pub enum SortableMsg {
    Reload,
    //DragStart(usize,)
}

impl Component for Sortable {
    type Message = SortableMsg;
    type Properties = SortableProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = (js_sys::Math::random() * 1_000_000.0) as usize;
        let w = window();
        let item_count = ctx.props().items.len();
        
        let doc = w.doc();
        let on_mouse_down = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            log!("mouse down");
            let x = e.client_x();
            let y = e.client_y();

            for i in 0..item_count {
                let fid = format!("sortable-{id}-{i}");
                let el = doc.get_element_by_id(&fid).unwrap();
                let rect = el.get_bounding_client_rect();
                if x >= rect.left() as i32 && x <= rect.right() as i32 && y >= rect.top() as i32 && y <= rect.bottom() as i32 {
                    log!("{i} is dragged");
                    return;
                }
            }
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref()).unwrap();

        let on_mouse_move = Closure::wrap(Box::new(move |_: web_sys::Event| {
            log!("mouse move");
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref()).unwrap();

        let on_mouse_up = Closure::wrap(Box::new(move |_: web_sys::Event| {
            log!("mouse up");
        }) as Box<dyn FnMut(_)>);

        Self {
            id,
            positions: None,
            ordered: (0..ctx.props().items.len()).collect(),
            on_mouse_down,
            on_mouse_move,
            on_mouse_up,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SortableMsg::Reload => {
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        *self = Self::create(ctx);
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if self.positions.is_some() { return }
        
        let mut rects = Vec::new();
        let document = window().doc();
        for i in 0..self.ordered.len() {
            let item = document.get_element_by_id(&format!("sortable-{}-{i}", self.id)).unwrap();
            rects.push(item.get_bounding_client_rect());
        }

        let mut y_min = usize::MAX;
        let mut width_max = usize::MIN;
        for rect in &rects {
            y_min = y_min.min(rect.y() as usize);
            width_max = width_max.max(rect.width() as usize);
        }
        
        self.positions = Some(Positions { y_min, width_max, rects });
        
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            sleep(Duration::from_secs(5)).await;
            link.send_message(SortableMsg::Reload); // TESTING ONLY: REMOVE BEFORE MERGE
        });
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let items = self.ordered.iter().map(|i| {
            let item = ctx.props().items.get(*i).unwrap();
            let fid = format!("sortable-{}-{}", self.id, i);
            let style = match &self.positions {
                Some(Positions { y_min, width_max: _, rects }) => {
                    let rect = rects.get(*i).unwrap();
                    let y = rect.y() as usize - y_min;
                    let width = rect.width() as usize;
                    let style = format!("position: absolute; top: {y}px; width: {width}px;");
                    style
                },
                _ => String::new(),
            };
            html! {
                <div class="sortable-item" id={fid} style={style}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20"><path d="M2 11h16v2H2zm0-4h16v2H2zm8 11l3-3H7l3 3zm0-16L7 5h6l-3-3z"/></svg>
                    {item}
                </div>
            }
        }).collect::<Html>();

        html! {
            <div class="sortable">
                {items}
            </div>
        }
    }
}

impl Drop for Sortable {
    fn drop(&mut self) {
        let w = window();
        let _ = w.remove_event_listener_with_callback("mousedown", self.on_mouse_down.as_ref().unchecked_ref());
        let _ = w.remove_event_listener_with_callback("mousemove", self.on_mouse_move.as_ref().unchecked_ref());
        let _ = w.remove_event_listener_with_callback("mouseup", self.on_mouse_up.as_ref().unchecked_ref());
    }
}
