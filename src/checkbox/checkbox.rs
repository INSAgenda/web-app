use crate::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CheckboxProps {
    pub message: String,
    pub checked: bool,
}

pub struct Checkbox {
    id: usize,
}

pub enum CheckboxMsg {
}

impl Component for Checkbox {
    type Message = CheckboxMsg;
    type Properties = CheckboxProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let id = (js_sys::Math::random() * 1_000_000.0) as usize;

        Self { id }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let fid = format!("checkbox-{}", self.id);
        html! {
            <div class="checkbox">
                <input type="checkbox" id={fid.clone()} checked={ctx.props().checked} />
                <label for={fid.clone()} class="checkbox-box"></label>
                <label for={fid.clone()}>{ctx.props().message.as_str()}</label>
            </div>
        }
    }
}