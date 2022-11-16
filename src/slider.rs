use crate::prelude::*;

pub fn width() -> usize {
    window().inner_width().unwrap().as_f64().unwrap() as usize
}

pub struct SliderManager {
    enabled: bool,
    start_pos: Option<i32>,
    day_container: Option<HtmlElement>,
    days_offset: Rc<Cell<i32>>,
    swift_next_callback: Closure<dyn FnMut()>,
    swift_prev_callback: Closure<dyn FnMut()>,
    has_moved: bool,
}

impl SliderManager {
    pub fn init(link: Scope<Agenda>, day_offset: i32) -> Rc<RefCell<SliderManager>> {
        // Create callbacks
        let days_offset = Rc::new(Cell::new(day_offset));

        let link2 = link.clone();
        let swift_next_callback = Closure::wrap(Box::new(move || {
            link2.send_message(AgendaMsg::Next);
        }) as Box<dyn FnMut()>);

        let link2 = link.clone();
        let swift_prev_callback = Closure::wrap(Box::new(move || {
            link2.send_message(AgendaMsg::Previous);
        }) as Box<dyn FnMut()>);

        // Create slider
        let slider = Rc::new(RefCell::new(SliderManager {
            enabled: false,
            start_pos: None,
            day_container: None,
            days_offset,
            swift_next_callback,
            swift_prev_callback,
            has_moved: false,
        }));
        if width() <= 1000 {
            slider.borrow_mut().enable();
        }
        let last_pos = Rc::new(Cell::new(0));
    
        let slider2 = Rc::clone(&slider);
        let link2 = link;
        let resize = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let mut slider = match slider2.try_borrow_mut() {
                Ok(slider) => slider,
                Err(_) => {
                    sentry_report("Slider could not be mutably borrowed");
                    return
                },
            };
            match slider.enabled {
                true if width() > 1000 => {
                    link2.send_message(AgendaMsg::Refresh);
                    slider.disable();
                }
                false if width() <= 1000 => {
                    link2.send_message(AgendaMsg::Refresh);
                    slider.enable();
                },
                _ => (),
            }
        }) as Box<dyn FnMut(_)>);
        let w = window();
        w.add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref()).unwrap();
        resize.forget();

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let move_animation = Rc::new(Closure::wrap(Box::new(move || {
            let _ = slider2.try_borrow_mut().map(|mut s| s.touch_move(last_pos2.get()));
        }) as Box<dyn FnMut()>));

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let end_animation = Rc::new(Closure::wrap(Box::new(move || {
            let _ = slider2.try_borrow_mut().map(|mut s| s.touch_end(last_pos2.get()));
        }) as Box<dyn FnMut()>));

        // Start

        let slider2 = Rc::clone(&slider);
        let mouse_down = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let mut slider = match slider2.try_borrow_mut() {
                Ok(slider) => slider,
                Err(_) => {
                    sentry_report("Slider could not be mutably borrowed");
                    return
                },
            };
            if slider.enabled {
                slider.touch_start(event.client_x() as i32, event.client_y() as i32);
            }
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("mousedown", mouse_down.as_ref().unchecked_ref()).unwrap();
        mouse_down.forget();

        let slider2 = Rc::clone(&slider);
        let touch_start = Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
            let mut slider = match slider2.try_borrow_mut() {
                Ok(slider) => slider,
                Err(_) => {
                    sentry_report("Slider could not be mutably borrowed");
                    return
                },
            };
            if slider.enabled {
                let mouse_x = event.touches().get(0).unwrap().client_x() as i32;
                let mouse_y = event.touches().get(0).unwrap().client_y() as i32;
                slider.touch_start(mouse_x, mouse_y);
            }
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("touchstart", touch_start.as_ref().unchecked_ref()).unwrap();
        touch_start.forget();

        // Move

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let move_animation2 = Rc::clone(&move_animation);
        let mouse_move = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            match slider2.try_borrow() {
                Ok(slider) => {
                    if slider.can_move() {
                        last_pos2.set(event.client_x() as i32);
                        window().request_animation_frame((*move_animation2).as_ref().unchecked_ref()).unwrap();
                    }
                },
                Err(_) => sentry_report("Can't borrow slider."),
            }
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref()).unwrap();
        mouse_move.forget();

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let move_animation2 = Rc::clone(&move_animation);
        let touch_move = Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
            if slider2.try_borrow().map(|s| s.can_move()).unwrap_or_default() {
                let mouse_x = event.touches().get(0).unwrap().client_x() as i32;
                last_pos2.set(mouse_x);
                window().request_animation_frame((*move_animation2).as_ref().unchecked_ref()).unwrap();
            }
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("touchmove", touch_move.as_ref().unchecked_ref()).unwrap();
        touch_move.forget();

        // End

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let end_animation2 = Rc::clone(&end_animation);
        let mouse_end = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if slider2.try_borrow().map(|s| s.can_move()).unwrap_or_default() {
                last_pos2.set(event.client_x() as i32);
                window().request_animation_frame((*end_animation2).as_ref().unchecked_ref()).unwrap();
            }
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("mouseup", mouse_end.as_ref().unchecked_ref()).unwrap();
        w.add_event_listener_with_callback("mouseleave", mouse_end.as_ref().unchecked_ref()).unwrap();
        mouse_end.forget();

        let slider2 = Rc::clone(&slider);
        let end_animation2 = Rc::clone(&end_animation);
        let touch_end = Closure::wrap(Box::new(move |_: web_sys::TouchEvent| {
            if slider2.try_borrow().map(|s| s.can_move()).unwrap_or_default() {
                window().request_animation_frame((*end_animation2).as_ref().unchecked_ref()).unwrap();
            }
        }) as Box<dyn FnMut(_)>);
        w.add_event_listener_with_callback("touchend", touch_end.as_ref().unchecked_ref()).unwrap();
        touch_end.forget();
    
        slider
    }
    
    pub fn enable(&mut self) {
        self.enabled = width() <= 1000;
        self.start_pos = None;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.start_pos = None;

        let doc = window().doc();
        if let Some(day_container) = doc.get_element_by_id("day-container").map(|e| e.dyn_into::<HtmlElement>().unwrap()) {
            if width() <= 1000 {
                day_container.style().set_property("right", &format!("{}%", self.days_offset.get().abs()*5)).unwrap();
            }
        }
    }

    fn get_cached_day_container(&mut self) -> Option<HtmlElement> {
        match &self.day_container {
            Some(day_container) => Some(day_container.clone()),
            None => {
                let doc = window().doc();
                let day_container = doc.get_element_by_id("day-container").map(|e| e.dyn_into::<HtmlElement>().unwrap())?;
                self.day_container = Some(day_container.clone());
                Some(day_container)
            }
        }
    }

    fn touch_start(&mut self, mouse_x: i32, mouse_y: i32) {
        let doc = window().doc();
        self.day_container = doc.get_element_by_id("day-container").map(|e| e.dyn_into().unwrap());
        self.start_pos = None;
        self.has_moved = false;
        if let Some(day_container) = &self.day_container {
            let rect = day_container.get_bounding_client_rect();
            if rect.y() as i32 <= mouse_y && mouse_y <= rect.y() as i32 + rect.height() as i32 {
                self.start_pos = Some(mouse_x);
            }
        }
    }

    fn can_move(&self) -> bool {
        self.enabled && self.start_pos.is_some() && self.day_container.is_some()
    }

    fn touch_move(&mut self, mouse_x: i32) {
        let day_container = match self.get_cached_day_container() {
            Some(day_container) => day_container,
            None => return,
        };
        let start_pos = match self.start_pos {
            Some(start_pos) => start_pos,
            None => return,
        };
        self.has_moved = true;
        let offset = mouse_x - start_pos;

        day_container.style().remove_property("transform").unwrap();
        day_container.style().set_property("position", "relative").unwrap();
        day_container.style().set_property("right", &format!("calc({}% + {}px)", self.days_offset.get().abs()*5, -offset)).unwrap();
    }

    fn touch_end(&mut self, mouse_x: i32) {
        let start_pos = match self.start_pos.take() {
            Some(start_pos) => start_pos,
            None => return,
        };
        let day_container = match self.get_cached_day_container() {
            Some(day_container) => day_container,
            None => return,
        };

        let offset = mouse_x - start_pos;
        if !self.has_moved{
            return;
        }
        if offset > 90 {
            day_container.style().set_property("right", &format!("{}%", self.days_offset.get().abs()*5)).unwrap();
            window().set_timeout_with_callback(self.swift_prev_callback.as_ref().unchecked_ref()).unwrap();
        } else if offset < -90 {
            day_container.style().set_property("right", &format!("{}%", self.days_offset.get().abs()*5)).unwrap();
            window().set_timeout_with_callback(self.swift_next_callback.as_ref().unchecked_ref()).unwrap();
        } else {
            day_container.style().set_property("right", &format!("{}%", self.days_offset.get().abs()*5)).unwrap();
        }
    }

    pub fn set_offset(&mut self, offset: i32) {
        let Some(day_container) = self.get_cached_day_container() else {return};

        if self.enabled && width() <= 1000 {
            self.days_offset.set(offset);

            day_container.style().set_property("right", &format!("{}%", self.days_offset.get().abs()*5)).unwrap();
        } 
    }
}
