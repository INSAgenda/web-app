use crate::prelude::*;

pub struct SurveyComp {
    progress: usize,
    answers: Vec<Option<Answer>>,
}

pub enum SurveyMsg {
    Back,
    Next,
    TextInput(InputEvent),
    CheckboxChange(bool),
    SelectChange(bool, usize),
    RadioChange(web_sys::Event, usize),
    ValueChange(web_sys::Event),
    PriorityChange(Vec<usize>),
}

#[derive(Properties)]
pub struct SurveyProps {
    pub survey: Rc<Survey>,
    pub app_link: Scope<App>,
}

impl PartialEq for SurveyProps {
    fn eq(&self, other: &Self) -> bool {
        self.survey.id == other.survey.id
    }
}

trait HackTraitGetLocalizedText {
    fn get_localized(&self, lang: &str) -> Option<String>;
}

impl HackTraitGetLocalizedText for HashMap<String, String> {
    fn get_localized(&self, lang: &str) -> Option<String> {
        if let Some(best) = self.get(lang).cloned() { return Some(best) }
        if let Some(default) = self.get("").cloned() { return Some(default) }
        if let Some(random_key) = self.keys().next() { return self.get(random_key).cloned() }
        None
    }
}

impl Component for SurveyComp {
    type Message = SurveyMsg;
    type Properties = SurveyProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            progress: 0,
            answers: ctx.props().survey.questions.iter().map(|s| match s.possible_answer {
                PossibleAnswer::Boolean { default } => Some(Answer::Boolean(default)),
                PossibleAnswer::Select(_) => Some(Answer::Select(Vec::new())), 
                _ => None // TODO: should priority have their default?
            }).collect(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SurveyMsg::Back => {
                if self.progress == 0 {
                    ctx.props().app_link.send_message(AppMsg::SetPage(Page::Agenda));
                    return false;
                }
                self.progress -= 1;
                true
            }
            SurveyMsg::Next => {
                if self.progress == ctx.props().survey.questions.len() {
                    ctx.props().app_link.send_message(AppMsg::SetPage(Page::Agenda));
                }
                self.progress += 1;
                true
            }
            SurveyMsg::TextInput(e) => {
                let target = e.target().unwrap();
                let target = target.dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
                let value = target.value();

                let was_some = self.answers[self.progress - 1].is_some(); // TODO also check max lenght
                if value.is_empty() {
                    self.answers[self.progress - 1] = None;
                    was_some
                } else {
                    self.answers[self.progress - 1] = Some(Answer::Input(value));
                    !was_some
                }
            }
            SurveyMsg::CheckboxChange(checked) => {
                self.answers[self.progress - 1] = Some(Answer::Boolean(checked));
                false
            }
            SurveyMsg::SelectChange(checked, i) => {
                let mut values = match self.answers[self.progress - 1].clone() {
                    Some(Answer::Select(old_values)) => old_values,
                    _ => Vec::new()
                };
                match checked {
                    true => values.push(i as u16),
                    false => values.retain(|oi| *oi != i as u16),
                }
                self.answers[self.progress - 1] = Some(Answer::Select(values));
                true
            }
            SurveyMsg::RadioChange(e, i) => {
                let target = e.target().unwrap();
                let target = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                if target.checked() {
                    self.answers[self.progress - 1].replace(Answer::Radio(i as u16)).is_none()
                } else {
                    false
                }
            }
            SurveyMsg::ValueChange(e) => {
                let target = e.target().unwrap();
                let target = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                let value = target.value();
                if let Ok(value) = value.parse() {
                    self.answers[self.progress - 1].replace(Answer::Value(value)).is_none()
                } else {
                    false
                }
            }
            SurveyMsg::PriorityChange(order) => {
                let order = order.into_iter().map(|i| i as u16).collect();
                self.answers[self.progress - 1].replace(Answer::Priority(order)).is_none()
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let l = SETTINGS.locale();
        let mut slides = Vec::new();

        for survey_question in &ctx.props().survey.questions {
            let question = survey_question.question.get_localized(l);
            let content = match survey_question.possible_answer {
                PossibleAnswer::Input { max_length, ref placeholder } => {
                    let value = match self.answers.get(self.progress.saturating_sub(1)).as_ref() {
                        Some(Some(Answer::Input(value))) => Some(value.to_string()),
                        _ => None,
                    };
                    html! {
                        <div class="survey-slide">
                            if let Some(question) = question {
                                <h2>{question}</h2>
                            }
                            <textarea
                                class="survey-input"
                                value={value}
                                rows="4"
                                maxlength={max_length.to_string()}
                                placeholder={placeholder.to_string()}
                                oninput={ctx.link().callback(SurveyMsg::TextInput)}
                            ></textarea>
                        </div>
                    }
                },
                PossibleAnswer::Boolean { default } => {
                    let checked = match self.answers.get(self.progress.saturating_sub(1)).as_ref() {
                        Some(Some(Answer::Boolean(value))) => *value,
                        _ => default,
                    };
                    html! {
                        <div class="survey-slide">
                            <Checkbox message={question.unwrap_or_default()} checked={checked} onchange={ctx.link().callback(SurveyMsg::CheckboxChange)} />
                        </div>
                    }
                },
                PossibleAnswer::Select(ref options) => {
                    let checked_list = match  self.answers.get(self.progress.saturating_sub(1)).as_ref() {
                        Some(Some(Answer::Select(values))) => values.clone(),
                        _ => Vec::new(),
                    };
                    let checkboxes = options.iter().enumerate().map(|(i, proposal)| {
                        let proposal = proposal.get_localized(l);
                        html! {
                            <Checkbox message={proposal.unwrap_or_default()} checked={checked_list.contains(&(i as u16))} onchange={ctx.link().callback(move |v| SurveyMsg::SelectChange(v, i))} />
                        }
                    }).collect::<Html>();
                    html! {
                        <div class="survey-slide">
                            if let Some(question) = question {
                                <h2>{question}</h2>
                            }
                            {checkboxes}
                        </div>
                    }
                },
                PossibleAnswer::Radio(ref options) => {
                    let checked = match self.answers.get(self.progress.saturating_sub(1)).as_ref() {
                        Some(Some(Answer::Radio(value))) => Some(*value),
                        _ => None,
                    };
                    let options = options.iter().enumerate().map(|(i, option)| {
                        let option = option.get_localized(l);
                        html! {
                            <label class="survey-radio">
                                <input type="radio" name="survey-radio" checked={Some(i as u16) == checked} onchange={ctx.link().callback(move |v| SurveyMsg::RadioChange(v, i))} />
                                <div>{option.unwrap_or_default()}</div>
                            </label>
                        }
                    }).collect::<Html>();
                    html! {
                        <div class="survey-slide">
                            if let Some(question) = question {
                                <h2>{question}</h2>
                            }
                            {options}
                        </div>
                    }
                },
                PossibleAnswer::Value { min, max, step } => {
                    let value = match self.answers.get(self.progress.saturating_sub(1)).as_ref() {
                        Some(Some(Answer::Value(value))) => Some(*value),
                        _ => None,
                    };
                    html! {
                        <div class="survey-slide">
                            if let Some(question) = question {
                                <h2>{question}</h2>
                            }
                            <input
                                type="range"
                                min={min.to_string()}
                                max={max.to_string()}
                                value={value.map(|v| v.to_string())}
                                step={step.to_string()}
                                onchange={ctx.link().callback(SurveyMsg::ValueChange)} />
                        </div>
                    }
                },
                PossibleAnswer::Priority(ref items) => {
                    let order = match self.answers.get(self.progress.saturating_sub(1)).as_ref() {
                        Some(Some(Answer::Priority(order))) => order.iter().map(|c| *c as usize).collect(),
                        _ => {
                            let mut indexes = (0..items.len()).collect::<Vec<_>>();
                            let mut order = Vec::new();
                            while !indexes.is_empty() {
                                let i = js_sys::Math::random() * (indexes.len()-1) as f64;
                                order.push(indexes.remove(i.round() as usize));
                            }
                            order
                        },
                    };
                    let items: Vec<_> = items.iter().map(|i| { i.get_localized(l).unwrap_or_default() }).collect();
                    html! {
                        <div class="survey-slide">
                            if let Some(question) = question {
                                <h2>{question}</h2>
                            }
                            <Sortable items={items} onchange={ctx.link().callback(SurveyMsg::PriorityChange)} order={order} />
                        </div>
                    }
                },
            };
            slides.push(content);
        }

        let opt_description = ctx.props().survey.description.get_localized(l);
        let survey_lenght = slides.len() + 1;

        let back_msg = if self.progress == 0 { "Fermer" } else { "Précédent" };
        let next_msg = if self.progress == 0 { "Commencer" } else if self.progress == survey_lenght - 1 { "Terminer" } else { "Suivant" };

        let next_disabled = (1..survey_lenght).contains(&self.progress) && ctx.props().survey.questions[self.progress - 1].required && match self.answers[self.progress - 1] {
            Some(Answer::Input(ref data)) => data.is_empty(),
            Some(_) => false,
            None => true,
        };

        template_html!(
            "src/survey/survey.html",
            title = {ctx.props().survey.title.as_str()},
            survey_lenght = {survey_lenght.to_string()},
            survey_progress = {self.progress.to_string()},
            slide_iter = {slides.into_iter()},
            onclick_next = {ctx.link().callback(|_| SurveyMsg::Next)},
            onclick_back = {ctx.link().callback(|_| SurveyMsg::Back)},
            ...
        )
    }
}
