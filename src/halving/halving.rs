use crate::prelude::*;

pub struct HalvingCountdown {
    remaining_seconds: i64,
    elapsed_seconds: i64,
    interval_handle: Option<i32>,
    emoji: char,
    message: &'static str,
}

pub enum HalvingCountdownMsg {
    Tick
}

impl Component for HalvingCountdown {
    type Message = HalvingCountdownMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let remaining_seconds = 1713603600 - now();

        let mut interval_handle = None;
        if remaining_seconds > -86400/2 {
            let link2 = ctx.link().clone();
            let closure = Closure::wrap(Box::new(move || {
                link2.send_message(HalvingCountdownMsg::Tick);
            }) as Box<dyn FnMut()>);
            interval_handle = window().set_interval_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), 1000).ok();
            closure.forget();
        }

        Self {
            remaining_seconds,
            elapsed_seconds: 0,
            interval_handle,
            emoji: match remaining_seconds % 5 {
                0 => '🎉',
                1 => '🚀',
                2 => '🔥',
                3 => '😍',
                4 => '💎',
                _ => unreachable!(),
            },
            message: match remaining_seconds % 3 {
                0 => "Vires In Numeris",
                1 => "We are all Satoshi",
                2 => "Don't trust, verify",
                _ => unreachable!()
            },
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HalvingCountdownMsg::Tick => {
                self.remaining_seconds -= 1;
                self.elapsed_seconds += 1;
                true
            }
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        if self.remaining_seconds < -86400/2 {
            return html!{};
        }

        if self.remaining_seconds < 86400/2 {
            let message = match self.elapsed_seconds < 10 {
                true => format!("Halving imminent ! {}", self.emoji),
                false => self.message.to_owned(),
            };
            return html! {
                <div id="open-calendar-halving-countdown">{message}</div>
            };
        }

        let days = self.remaining_seconds.div_euclid(86400);
        let hours = (self.remaining_seconds - days * 86400).div_euclid(3600);
        let minutes = (self.remaining_seconds - days * 86400 - hours * 3600).div_euclid(60);
        let seconds = self.remaining_seconds - days * 86400 - hours * 3600 - minutes * 60;
        let message = match days {
            0 => format!("Halving in {hours:02.}:{minutes:02.}:{seconds:02.}"),
            1 => format!("Halving in 1 day {hours:02.}:{minutes:02.}:{seconds:02.}"),
            _ => format!("Halving in {days} days {hours:02.}:{minutes:02.}:{seconds:02.}"),
        };
        html! {
            <div id="open-calendar-halving-countdown">{message}</div>
        }
    }
}

impl Drop for HalvingCountdown {
    fn drop(&mut self) {
        log!("dropped countdown");
        if let Some(interval_handle) = self.interval_handle {
            window().clear_interval_with_handle(interval_handle);
        }
    }
}
