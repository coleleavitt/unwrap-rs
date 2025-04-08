use super::YewComponent;
use yew::prelude::*;
use gloo_timers::callback::Timeout;
use std::rc::Rc;
use std::cell::RefCell;

/// Radiation-hardened typing animation
pub struct TypingAnimation {
    text: Rc<RefCell<String>>,
    target_text: String,
    char_index: Rc<RefCell<usize>>,
    is_complete: Rc<RefCell<bool>>,
}

pub enum Msg {
    Tick,
    Reset,
}

impl YewComponent for TypingAnimation {
    fn render(&self) -> Html {
        html! {
            <div class="typing-animation-container">
                <span class="base-text">{"Result"}</span>
                <span class="typing-text">{self.get_current_text()}</span>
                <span class="cursor-blink">{"_"}</span>
            </div>
        }
    }

    fn create_component() -> Self {
        TypingAnimation {
            text: Rc::new(RefCell::new(String::new())),
            target_text: ".unwrap()".to_string(),
            char_index: Rc::new(RefCell::new(0)),
            is_complete: Rc::new(RefCell::new(false)),
        }
    }
}

impl Component for TypingAnimation {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let component = Self::create_component();
        component.schedule_tick(ctx);
        component
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                let mut index = self.char_index.borrow_mut();

                if *index < self.target_text.len() {
                    if let Some(next_char) = self.target_text.chars().nth(*index) {
                        let mut text = self.text.borrow_mut();
                        text.push(next_char);
                        *index = index.saturating_add(1);
                        self.schedule_tick(ctx);
                    }
                } else {
                    *self.is_complete.borrow_mut() = true;
                    self.schedule_reset(ctx);
                }
                true
            },
            Msg::Reset => {
                *self.text.borrow_mut() = String::new();
                *self.char_index.borrow_mut() = 0;
                *self.is_complete.borrow_mut() = false;
                self.schedule_tick(ctx);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.render()
    }
}

impl TypingAnimation {
    fn schedule_tick(&self, ctx: &Context<Self>) {
        const TICK_DELAY_MS: u32 = 150;

        let link = ctx.link().clone();
        let timeout = Timeout::new(TICK_DELAY_MS, move || {
            link.send_message(Msg::Tick);
        });
        timeout.forget();
    }

    fn schedule_reset(&self, ctx: &Context<Self>) {
        const RESET_DELAY_MS: u32 = 3000;

        let link = ctx.link().clone();
        let timeout = Timeout::new(RESET_DELAY_MS, move || {
            link.send_message(Msg::Reset);
        });
        timeout.forget();
    }

    fn get_current_text(&self) -> String {
        self.text.borrow().clone()
    }
}
