use crate::prelude::*;

#[derive(Properties, PartialEq, Eq)]
pub struct SortableProps {
    pub items: Vec<String>,
}

pub struct Sortable {
    id: usize,
    positions: Option<(usize, Vec<usize>)>,
    ordered: Vec<usize>,
}

pub enum SortableMsg {
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

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        *self = Self::create(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut offset = 0;
        let items = self.ordered.iter().map(|i| {
            let item = ctx.props().items.get(*i).unwrap();
            let fid = format!("sortable-{}-{}", self.id, i);
            let style = match &self.positions {
                Some(positions) => {
                    let style = format!("position: absolute; top: {offset}px;");
                    offset += positions.1.get(*i).unwrap();
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
