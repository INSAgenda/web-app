use crate::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CheckboxProps {
    pub message: String,
    pub checked: bool,
    #[prop_or_default]
    pub onchange: Option<Callback<bool>>,
}

pub struct Checkbox {
    id: usize,
}

pub enum CheckboxMsg {
    Change(web_sys::Event)
}

impl Component for Checkbox {
    type Message = CheckboxMsg;
    type Properties = CheckboxProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let id = (js_sys::Math::random() * 1_000_000.0) as usize;

        Self { id }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CheckboxMsg::Change(event) => if let Some(onchange) = ctx.props().onchange.as_ref() {
                let target = event.target().unwrap();
                let target = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                onchange.emit(target.checked());
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let fid = format!("checkbox-{}", self.id);
        html! {
            <div class="checkbox">
                <input type="checkbox" id={fid.clone()} checked={ctx.props().checked} onchange={ctx.link().callback(CheckboxMsg::Change)} />
                <label for={fid.clone()} class="checkbox-box"></label>
                <label for={fid.clone()}>{ctx.props().message.as_str()}</label>
            </div>
        }
    }
}