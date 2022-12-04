use crate::prelude::*;

#[derive(Properties, PartialEq, Eq)]
pub struct SortableProps {
    pub items: Vec<String>,
}


struct Positions {
    start_y: usize,
    width: usize,
    heights: Vec<usize>,
}

pub struct Sortable {
    id: usize,
    positions: Option<Positions>,
    ordered: Vec<usize>,
}

pub enum SortableMsg {
    Reload
}

impl Component for Sortable {
    type Message = SortableMsg;
    type Properties = SortableProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = (js_sys::Math::random() * 1_000_000.0) as usize;

        Self {
            id,
            positions: None,
            ordered: (0..ctx.props().items.len()).collect(),
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
        
        let mut heights = Vec::new();
        let mut start_y = usize::MAX;
        let mut width = 0;
        let document = window().doc();
        for i in 0..self.ordered.len() {
            let item = document.get_element_by_id(&format!("sortable-{}-{i}", self.id)).unwrap();
            let rect = item.get_bounding_client_rect();
            heights.push(rect.height() as usize);
            start_y = start_y.min(rect.x() as usize);
            width = width.max(rect.width() as usize);
        }
        
        self.positions = Some(Positions { start_y, width, heights });
        
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            sleep(Duration::from_secs(5)).await;
            link.send_message(SortableMsg::Reload); // TESTING ONLY: REMOVE BEFORE MERGE
        });
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut offset = 0;
        let items = self.ordered.iter().map(|i| {
            let item = ctx.props().items.get(*i).unwrap();
            let fid = format!("sortable-{}-{}", self.id, i);
            let style = match &self.positions {
                Some(Positions { start_y: _, width, heights }) => {
                    let style = format!("position: absolute; top: calc({offset}px + {i} * .5rem); width: calc({width}px - 1rem);");
                    offset += heights.get(*i).unwrap();
                    style
                },
                None => String::new(),
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
