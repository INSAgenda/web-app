use crate::prelude::*;

pub enum Flag {
    Palestine,
    Ukraine,
    Kanaky,
    HongKong,
    Taiwan,
    // TODO: Tibet
}

impl Component for Flag {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let crypto = window().crypto().unwrap();
        let mut random_bytes = [0; 1];
        crypto
            .get_random_values_with_u8_array(&mut random_bytes)
            .unwrap();
        let random_value = random_bytes[0] as f32 / 255.0;

        match random_value {
            0.0..0.40 => Self::Palestine, // 40%
            0.40..0.70 => Self::Ukraine,  // 30%
            0.70..0.85 => Self::Kanaky,   // 15%
            0.85..0.95 => Self::HongKong, // 10%
            _ => Self::Taiwan,            // 5%
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let (href, alt, svg) = match self {
            Flag::Palestine => ("https://donate.unrwa.org/int/en/general", "Soutenez le peuple palestinien", "/agenda/images/palestine.svg"),
            Flag::Ukraine => ("https://war.ukraine.ua/donate/", "Soutenez l'Ukraine", "/agenda/images/ukraine.svg"),
            Flag::Kanaky => ("https://www.amnesty.org/fr/latest/news/2024/05/kanaky-new-caledonia-french-authorities-must-uphold-rights-of-the-indigenous-kanak-people-amid-unrest/", "Drapeau kanak", "/agenda/images/kanaky.svg"),
            Flag::HongKong => ("https://youtu.be/9iF7xiEU5c8", "Drapeau de Hong Kong", "/agenda/images/hongkong.svg"),
            Flag::Taiwan => ("https://spiritofamerica.org/project/help-prepare-taiwan-for-an-emergency",
            "Drapeau de Taïwan", "/agenda/images/taiwan.svg"),
        };

        html! {
            <a id="flag-link" href={ href }>
                <img src={svg} alt={ alt } />
            </a>
        }
    }
}
