use crate::log;
use wasm_bindgen::{prelude::*, JsCast};
use std::{rc::Rc, cell::{Cell, RefCell}};

fn width() -> usize {
    web_sys::window().unwrap().inner_width().unwrap().as_f64().unwrap() as usize
}

pub struct SliderManager {
    enabled: bool,
    selected_index: u32,
    start_pos: Option<i32>,
    day_container: Option<web_sys::HtmlElement>,
}

impl SliderManager {
    pub fn init() -> Rc<RefCell<SliderManager>> {
        let slider = Rc::new(RefCell::new(SliderManager {
            enabled: false,
            start_pos: None,
            selected_index: 0,
            day_container: None,
        }));
        if width() <= 1000 {
            slider.borrow_mut().enable();
        }
        let last_pos = Rc::new(Cell::new(0));
    
        let slider2 = Rc::clone(&slider);
        let resize = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let mut slider = slider2.borrow_mut();
            match slider.enabled {
                true if width() > 1000 => {
                    slider.disable();
                }
                false if width() <= 1000 => {
                    slider.enable();
                },
                _ => (),
            }
        }) as Box<dyn FnMut(_)>);
        let window = web_sys::window().unwrap();
        window.add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref()).unwrap();
        resize.forget();

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let move_animation = Rc::new(Closure::wrap(Box::new(move || {
            slider2.borrow_mut().touch_move(last_pos2.get());
        }) as Box<dyn FnMut()>));

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let end_animation = Rc::new(Closure::wrap(Box::new(move || {
            slider2.borrow_mut().touch_end(last_pos2.get());
        }) as Box<dyn FnMut()>));

        // Start

        let slider2 = Rc::clone(&slider);
        let mouse_down = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let mut slider = slider2.borrow_mut();
            if slider.enabled {
                slider.touch_start(event.client_x() as i32);
            }
        }) as Box<dyn FnMut(_)>);
        let window = web_sys::window().unwrap();
        window.add_event_listener_with_callback("mousedown", mouse_down.as_ref().unchecked_ref()).unwrap();
        mouse_down.forget();

        let slider2 = Rc::clone(&slider);
        let touch_start = Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
            let mut slider = slider2.borrow_mut();
            if slider.enabled {
                let mouse_x = event.touches().get(0).unwrap().client_x() as i32;
                slider.touch_start(mouse_x);
            }
        }) as Box<dyn FnMut(_)>);
        let window = web_sys::window().unwrap();
        window.add_event_listener_with_callback("touchstart", touch_start.as_ref().unchecked_ref()).unwrap();
        touch_start.forget();

        // Move

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let move_animation2 = Rc::clone(&move_animation);
        let mouse_move = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if slider2.borrow().can_move() {
                last_pos2.set(event.client_x() as i32);
                let window = web_sys::window().unwrap();
                window.request_animation_frame((*move_animation2).as_ref().unchecked_ref()).unwrap();
            }
        }) as Box<dyn FnMut(_)>);
        let window = web_sys::window().unwrap();
        window.add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref()).unwrap();
        mouse_move.forget();

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let move_animation2 = Rc::clone(&move_animation);
        let touch_move = Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
            if slider2.borrow().can_move() {
                let mouse_x = event.touches().get(0).unwrap().client_x() as i32;
                last_pos2.set(mouse_x);
                let window = web_sys::window().unwrap();
                window.request_animation_frame((*move_animation2).as_ref().unchecked_ref()).unwrap();
            }
        }) as Box<dyn FnMut(_)>);
        let window = web_sys::window().unwrap();
        window.add_event_listener_with_callback("touchmove", touch_move.as_ref().unchecked_ref()).unwrap();
        touch_move.forget();

        // End

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let end_animation2 = Rc::clone(&end_animation);
        let mouse_end = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if slider2.borrow().can_move() {
                last_pos2.set(event.client_x() as i32);
                let window = web_sys::window().unwrap();
                window.request_animation_frame((*end_animation2).as_ref().unchecked_ref()).unwrap();
            }
        }) as Box<dyn FnMut(_)>);
        let window = web_sys::window().unwrap();
        window.add_event_listener_with_callback("mouseup", mouse_end.as_ref().unchecked_ref()).unwrap();
        window.add_event_listener_with_callback("mouseleave", mouse_end.as_ref().unchecked_ref()).unwrap();
        mouse_end.forget();

        let slider2 = Rc::clone(&slider);
        let last_pos2 = Rc::clone(&last_pos);
        let end_animation2 = Rc::clone(&end_animation);
        let touch_end = Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
            if slider2.borrow().can_move() {
                let window = web_sys::window().unwrap();
                window.request_animation_frame((*end_animation2).as_ref().unchecked_ref()).unwrap();
            }
        }) as Box<dyn FnMut(_)>);
        let window = web_sys::window().unwrap();
        window.add_event_listener_with_callback("touchend", touch_end.as_ref().unchecked_ref()).unwrap();
        touch_end.forget();
    
        slider
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
        self.start_pos = None;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.start_pos = None;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        if let Some(day_container) = document.get_element_by_id("day-container").map(|e| e.dyn_into::<web_sys::HtmlElement>().unwrap()) {
            day_container.style().set_property("transform", "translateX(0px)").unwrap();
        }
    }

    fn touch_start(&mut self, mouse_x: i32) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        self.day_container = document.get_element_by_id("day-container").map(|e| e.dyn_into().unwrap());

        self.start_pos = match self.day_container.is_some() {
            true => Some(mouse_x),
            false => None,
        }
    }

    fn can_move(&self) -> bool {
        self.enabled && self.start_pos.is_some() && self.day_container.is_some()
    }

    fn touch_move(&mut self, mouse_x: i32) {
        let day_container = match self.day_container {
            Some(ref element) => element,
            None => return,
        };
        let start_pos = match self.start_pos {
            Some(start_pos) => start_pos,
            None => return,
        };

        let offset = mouse_x - start_pos;

        day_container.style().set_property("transform", &format!("translateX(calc(-20% * {} + {}px))", self.selected_index, offset)).unwrap();
    }

    fn touch_end(&mut self, mouse_x: i32) {
        let day_container = match self.day_container {
            Some(ref element) => element,
            None => return,
        };
        let start_pos = match self.start_pos {
            Some(start_pos) => start_pos,
            None => return,
        };

        let offset = mouse_x - start_pos;

        if offset > 90 && self.selected_index > 0 {
            self.selected_index -= 1;
        } else if offset < -90 && self.selected_index < 4 {
            self.selected_index += 1
        }

        day_container.style().set_property("transform", &format!("translateX(calc(-20% * {}))", self.selected_index)).unwrap();
        self.start_pos = None;
    }
}