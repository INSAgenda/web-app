use crate::prelude::*;

#[derive(Properties)]
pub struct SortableProps {
    pub items: Vec<String>,
    #[prop_or_default]
    pub order: Option<Vec<usize>>,
    #[prop_or_default]
    pub onchange: Option<Callback<Vec<usize>>>,
}

impl PartialEq for SortableProps {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items && self.order == other.order
    }
}

pub struct Sortable {
    /// The order indexes of items in properties.
    order: Rc<RefCell<Vec<usize>>>,
    
    /// Unique ID of this component
    id: usize,
    
    /// Yew doesn't update elements when it thinks they're the same. Except we modify the style property by hand so we need to add a key that changes every time we want yew to update the element.
    reload_id: usize,

    // Event handlers
    on_mouse_down: wasm_bindgen::prelude::Closure<dyn std::ops::FnMut(web_sys::MouseEvent)>,
    on_touch_start: wasm_bindgen::prelude::Closure<dyn std::ops::FnMut(web_sys::TouchEvent)>,
    on_mouse_move: wasm_bindgen::prelude::Closure<dyn std::ops::FnMut(web_sys::MouseEvent)>,
    on_touch_move: wasm_bindgen::prelude::Closure<dyn std::ops::FnMut(web_sys::TouchEvent)>,
    on_mouse_up: wasm_bindgen::prelude::Closure<dyn std::ops::FnMut(web_sys::Event)>,
}

pub enum SortableMsg {
    ChangeOrder(Vec<usize>),
}

impl Component for Sortable {
    type Message = SortableMsg;
    type Properties = SortableProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = (js_sys::Math::random() * 1_000_000.0) as usize;
        let w = window();
        let item_count = ctx.props().items.len();
        let order: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(match &ctx.props().order {
            Some(order) if order.len() == item_count => order.to_owned(),
            _ => (0..item_count).collect()
        }));
        let currently_dragged = Rc::new(RefCell::new(None));

        // Closure to release the currently dragged item
        let currently_dragged2 = currently_dragged.clone();
        let doc = w.doc();
        let link2 = ctx.link().clone();
        let release_drag = move || {
            if currently_dragged2.borrow_mut().take().is_some() {
                let mut rects = Vec::new();
                for i in 0..item_count {
                    let fid = format!("sortable-{id}-{i}");
                    let el = doc.get_element_by_id(&fid).unwrap();
                    let rect = el.get_bounding_client_rect();
                    rects.push((i, rect));
                }
                rects.sort_by_key(|(_, rect)| (rect.top() as i32 + rect.height() as i32) / 2);
                let new_order = rects.into_iter().map(|(i, _)| i).collect::<Vec<_>>();
                link2.send_message(SortableMsg::ChangeOrder(new_order));
            }
        };
        
        // On press (dragging starts)
        let doc = w.doc();
        let currently_dragged2 = currently_dragged.clone();
        let release_drag2 = release_drag.clone();
        let on_drag = move |x: i32, y: i32| {
            release_drag2();

            // Create a snapshot of element positions when dragging starts
            let mut centers = Vec::new();
            let mut dragged = None;
            for i in 0..item_count {
                let fid = format!("sortable-{id}-{i}");
                let el = doc.get_element_by_id(&fid).unwrap();
                let rect = el.get_bounding_client_rect();
                let top = rect.top() as i32;
                let bottom = rect.bottom() as i32;
                centers.push((bottom + top) / 2);

                // Detect which element has been dragged
                if x >= rect.left() as i32 && x <= rect.right() as i32 && y >= top && y <= bottom {
                    dragged = Some(i);
                }
            }
            if let Some(dragged) = dragged {
                currently_dragged2.borrow_mut().replace((dragged, y, centers));
            }
        };
        let on_drag2 = on_drag.clone();
        let on_mouse_down = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            on_drag2(e.client_x(), e.client_y());
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref()).unwrap();
        let on_touch_start = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            let touch = e.touches().get(0).unwrap();
            on_drag(touch.client_x(), touch.client_y());
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("touchstart", on_touch_start.as_ref().unchecked_ref()).unwrap();

        // On move events (dragging)
        let doc = w.doc();
        let ordered2 = order.clone();
        let on_move = move |y: i32| {
            if let Some((i, start_y, centers)) = currently_dragged.borrow().as_ref() {
                // Offset the dragged element by how much the cursor has moved
                let dy = y - start_y;
                let fid = format!("sortable-{id}-{i}");
                let el = doc.get_element_by_id(&fid).unwrap();
                el.set_attribute("style", &format!("transition: scale .2s ease; top: {dy}px; scale: 1.05; z-index: 999;")).unwrap();

                let rect = el.get_bounding_client_rect();
                let top = rect.top() as i32;
                let bottom = rect.bottom() as i32;
                let height = bottom - top;

                // Offset all other elements depending on if the dragged element has moved above or below them
                let position = ordered2.borrow().deref().iter().position(|&x| x == *i).unwrap();
                for other in 0..item_count {
                    let other_position = ordered2.borrow().deref().iter().position(|&x| x == other).unwrap();
                    if other_position == position { continue; }
                    let other_item_el = doc.get_element_by_id(&format!("sortable-{id}-{other}")).unwrap();
                    let center = centers.get(other).unwrap();

                    if other_position > position && bottom > *center {
                        other_item_el.set_attribute("style", &format!("top: calc(-{height}px - 0.5rem);")).unwrap();
                    } else if other_position < position && top < *center {
                        other_item_el.set_attribute("style", &format!("top: calc({height}px + 0.5rem);")).unwrap();
                    } else {
                        other_item_el.set_attribute("style", "top: 0px;").unwrap();
                    }
                }
            }
        };
        let on_move2 = on_move.clone();
        let on_mouse_move = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            on_move2(e.client_y());
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref()).unwrap();
        let on_touch_move = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            on_move(e.touches().get(0).unwrap().client_y());
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("touchmove", on_touch_move.as_ref().unchecked_ref()).unwrap();

        // On release events (dragging stops)
        let on_mouse_up = Closure::wrap(Box::new(move |_: web_sys::Event| {
            release_drag();
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref()).unwrap();
        w.add_event_listener_with_callback("mouseleave", on_mouse_up.as_ref().unchecked_ref()).unwrap();
        w.add_event_listener_with_callback("touchend", on_mouse_up.as_ref().unchecked_ref()).unwrap();

        Self {
            id,
            order,
            reload_id: 0,
            on_mouse_down,
            on_touch_start,
            on_mouse_move,
            on_touch_move,
            on_mouse_up,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SortableMsg::ChangeOrder(order) => {
                if let Some(onchange) = ctx.props().onchange.as_ref() {
                    onchange.emit(order.clone());
                }
                self.order.replace(order);
                self.reload_id += 1;
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        *self = Component::create(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let items = self.order.borrow().iter().map(|i| {
            let item = ctx.props().items.get(*i).unwrap();
            let fid = format!("sortable-{}-{}", self.id, i);
            html! {
                <div class="sortable-item" id={fid} style={format!("top: 0px; transition: unset; reload-id: {};", self.reload_id)}>
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
        let _ = w.remove_event_listener_with_callback("touchstart", self.on_touch_start.as_ref().unchecked_ref());
        let _ = w.remove_event_listener_with_callback("mousemove", self.on_mouse_move.as_ref().unchecked_ref());
        let _ = w.remove_event_listener_with_callback("touchmove", self.on_touch_move.as_ref().unchecked_ref());
        let _ = w.remove_event_listener_with_callback("mouseup", self.on_mouse_up.as_ref().unchecked_ref());
        let _ = w.remove_event_listener_with_callback("mouseleave", self.on_mouse_up.as_ref().unchecked_ref());
        let _ = w.remove_event_listener_with_callback("touchend", self.on_mouse_up.as_ref().unchecked_ref());
    }
}
