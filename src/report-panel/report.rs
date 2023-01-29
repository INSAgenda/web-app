use crate::prelude::*;

pub enum ReportReason {
    None,
    Harassment,
    Dox,
    Nsfw,
    Spam,
    Illegal,
    Unwanted,
    Other,
}

pub struct ReportPanel {
    selected: ReportReason,
}

pub enum ReportPanelMsg {
    Change,
}

impl Component for ReportPanel {
    type Message = ReportPanelMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            selected: ReportReason::None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().callback(|_| ReportPanelMsg::None);
        template_html! {
            "src/report-panel/report.html",
            
        }
    }
}
