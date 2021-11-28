use yew::prelude::*;
use std::rc::Rc;
use crate::log;

pub enum Msg {
    Init,
    Select(usize),
}

#[derive(Properties, Clone)]
pub struct GliderSelectorProp {
    pub values: Vec<&'static str>,
}

pub struct GliderSelector {
    values: Vec<&'static str>,
    sizes: Vec<u32>,
    selected: usize,
    id: String,
    link: Rc<ComponentLink<Self>>,
}

impl Component for GliderSelector {
    type Message = Msg;
    type Properties = GliderSelectorProp;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut id = String::from("glider-selector-");
        let crypto = web_sys::window().unwrap().crypto().unwrap();
        let mut random_bytes = [0; 20];
        crypto.get_random_values_with_u8_array(&mut random_bytes).unwrap();
        for byte in random_bytes {
            id.push((97 + (byte % (123 - 97))) as char);
        }

        GliderSelector {
            values: props.values,
            id,
            sizes: Vec::new(),
            selected: 0,
            link: Rc::new(link),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Init => {
                let this = web_sys::window().unwrap().document().unwrap().get_element_by_id(&self.id).unwrap();
                let children = this.children();
                self.sizes.clear();
                for i in 1..=self.values.len() {
                    let child = children.item(i as u32).unwrap();
                    let width = child.client_width();
                    self.sizes.push(width as u32);
                }
                true
            }
            Msg::Select(index) => {
                log!("select");
                self.selected = index;
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.values = props.values;
        true
    }

    fn view(&self) -> Html {
        let glider_selected = if self.sizes.len() == self.values.len() {
            let mut offset = 0;
            for i in 0..self.selected {
                offset += self.sizes[i];
            }
            let width = self.sizes[self.selected];

            html! {
                <div id="glider-selected" style=format!("left: {}px; width: calc({}px - 2rem);", offset, width)></div>
            }
        } else {
            let link2 = Rc::clone(&self.link);
            wasm_bindgen_futures::spawn_local(async move {
                crate::sleep(std::time::Duration::from_millis(100)).await;
                link2.send_message(Msg::Init);
            });

            html! {
                <div id="glider-selected"></div>
            }
        };

        html! {
            <div class="glider-selector" id=self.id.clone()>
                {glider_selected}
                {
                    self.values.iter().enumerate().map(|(i, v)|
                        if i == self.selected {
                            html! { <div style="color: white;">{v}</div> }
                        } else {
                            html! { <div onclick=self.link.callback(move |_| Msg::Select(i))>{v}</div> }
                        }
                    ).collect::<Html>()
                }
            </div>
        }
    }
}