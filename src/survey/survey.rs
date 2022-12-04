use crate::prelude::*;

pub struct SurveyComp {
    progress: usize,
}

pub enum SurveyMsg {
    Next,
}

#[derive(Properties, PartialEq)]
pub struct SurveyProps {
    pub survey: Rc<Survey>,
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

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            progress: 0,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SurveyMsg::Next => {
                self.progress += 1;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let l = SETTINGS.locale();
        let mut slides = Vec::new();

        for survey_question in &ctx.props().survey.questions {
            let question = survey_question.question.get_localized(l);
            let content = match survey_question.possible_answer {
                PossibleAnswer::Input { max_length } => html! {
                    <div class="survey-slide">
                        if let Some(question) = question {
                            <h2>{question}</h2>
                        }
                        <textarea class="survey-input" rows="4" maxlength={max_length.to_string()}></textarea>
                    </div>
                },
                PossibleAnswer::Checkbox => html! {
                    <div class="survey-slide">
                        <Checkbox message={question.unwrap_or_default()} checked={false} />
                    </div>
                },
                PossibleAnswer::MultipleChoice(ref options) => {
                    let options = options.iter().map(|option| {
                        let option = option.get_localized(l);
                        html! {
                            <option>{option.unwrap_or_default()}</option>
                        }
                    }).collect::<Html>();
                    html! {
                        <div class="survey-slide">
                            if let Some(question) = question {
                                <h2>{question}</h2>
                            }
                            <select class="survey-input">{options}</select>
                        </div>
                    }
                },
                PossibleAnswer::OneChoice(ref options) => {
                    let options = options.iter().map(|option| {
                        let option = option.get_localized(l);
                        html! {
                            <label class="survey-radio">
                                <input type="radio" name="survey-radio" />
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
                    html! {
                        <div class="survey-slide">
                            if let Some(question) = question {
                                <h2>{question}</h2>
                            }
                            <input type="range" min={min.to_string()} max={max.to_string()} step={step.to_string()} />
                        </div>
                    }
                },
                PossibleAnswer::Priority(ref items) => {
                    let items: Vec<_> = items.iter().map(|i| { i.get_localized(l).unwrap_or_default() }).collect();

                    html! {
                        <div class="survey-slide">
                            if let Some(question) = question {
                                <h2>{question}</h2>
                            }
                            <Sortable items={items} />
                        </div>
                    }
                },
            };
            slides.push(content);
        }

        let opt_description = ctx.props().survey.description.get_localized(l);
        let survey_lenght = slides.len() + opt_description.is_some() as usize;
        template_html!(
            "src/survey/survey.html",
            title = {ctx.props().survey.title.as_str()},
            survey_lenght = {survey_lenght.to_string()},
            survey_progress = {self.progress.to_string()},
            slide_iter = {slides.into_iter()},
            onclick_next = {ctx.link().callback(|_| SurveyMsg::Next)},
            ...
        )
    }
}
