pub use crate::prelude::*;

pub struct NotificationsPage;

pub enum NotificationsMsg {
}

impl Component for NotificationsPage {
    type Message = NotificationsMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let unseen_texts = vec!["I will write some great placeholder calling for a total and complete shutdown of ", "et cela"];
        let unseen_text_iter = unseen_texts.iter();
        let unseen_src_iter = unseen_texts.iter().map(|text| format!("https://api.dicebear.com/5.x/micah/svg?seed={text}", text = text.replace(" ", "+")));
        let unseen_alt_iter = unseen_texts.iter().map(|text| format!("Avatar of {text}"));
        let unseen_button_iter = unseen_texts.iter().map(|text| if 1 == 2 {Some(format!("Voir {text}", text = text))} else {None});

        let texts = vec!["I will write some great placeholder calling for a total and complete shutdown of ", "et cela"];
        let text_iter = texts.iter();
        let src_iter = texts.iter().map(|text| format!("https://api.dicebear.com/5.x/micah/svg?seed={text}", text = text.replace(" ", "+")));
        let alt_iter = texts.iter().map(|text| format!("Avatar of {text}"));
        let button_iter = texts.iter().map(|text| if 1 == 2 {Some(format!("Voir {text}", text = text))} else {None});

        template_html!("src/notifications/notifications.html", ...)
    }
}
