use crate::prelude::*;

pub enum Msg {
    Init,
    Select(usize),
}

#[derive(Properties, PartialEq, Clone)]
pub struct GliderSelectorProps {
    pub values: Vec<&'static str>,
    pub selected: usize,
    #[prop_or_default]
    pub on_change: Option<Callback<usize>>,
}

pub struct GliderSelector {
    sizes: Vec<u32>,
    selected: usize,
    id: String,
}

impl Component for GliderSelector {
    type Message = Msg;
    type Properties = GliderSelectorProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut id = String::from("glider-selector-");
        let crypto = window().crypto().unwrap();
        let mut random_bytes = [0; 20];
        crypto.get_random_values_with_u8_array(&mut random_bytes).unwrap();
        for byte in random_bytes {
            id.push((97 + (byte % (123 - 97))) as char);
        }

        GliderSelector {
            id,
            sizes: Vec::new(),
            selected: ctx.props().selected,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Init => {
                let this = window().document().unwrap().get_element_by_id(&self.id).unwrap();
                let children = this.children();
                self.sizes.clear();
                for i in 1..=ctx.props().values.len() {
                    let child = children.item(i as u32).unwrap();
                    let width = child.client_width();
                    self.sizes.push(width as u32);
                }
                true
            }
            Msg::Select(index) => {
                self.selected = index;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let glider_selected = if self.sizes.len() == ctx.props().values.len() {
            let mut offset = 0;
            for i in 0..self.selected {
                offset += self.sizes[i];
            }
            let width = self.sizes[self.selected];

            html! {
                <div id="glider-selected" style={format!("left: {}px; width: calc({}px - 2rem);", offset, width)}></div>
            }
        } else {
            let link2 = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                crate::sleep(std::time::Duration::from_millis(30)).await;
                link2.send_message(Msg::Init);
            });

            html! {
                <div id="glider-selected"></div>
            }
        };

        html! {
            <div class="glider-selector" id={self.id.clone()}>
                {glider_selected}
                {
                    ctx.props().values.iter().enumerate().map(|(i, v)|
                        if i == self.selected {
                            html! { <div style="color: white;">{v}</div> }
                        } else {
                            html! { <div onclick={
                                let on_change = ctx.props().on_change.clone();
                                ctx.link().callback(move |_| {
                                    if let Some(on_change) = &on_change {
                                        on_change.emit(i);
                                    }
                                    Msg::Select(i)
                                })
                            }>{v}</div> }
                        }
                    ).collect::<Html>()
                }
            </div>
        }
    }
}