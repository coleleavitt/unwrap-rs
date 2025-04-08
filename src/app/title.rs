use super::{YewComponent, StyledComponent, Unwrappable, UnwrappableComponent};
use yew::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Title {
    unwrapped: Rc<RefCell<bool>>,
}

impl YewComponent for Title {
    fn render(&self) -> Html {
        self.render_with_unwrap_state()
    }

    fn create_component() -> Self {
        Title {
            unwrapped: Rc::new(RefCell::new(false)),
        }
    }
}

impl StyledComponent for Title {
    fn base_classes(&self) -> Vec<String> {
        vec!["text-3xl".into(), "font-bold".into()]
    }

    fn conditional_classes(&self) -> Vec<(String, bool)> {
        vec![
            ("transition-all".into(), true),
            ("duration-300".into(), true),
            ("transform".into(), true),
            ("scale-110".into(), *self.unwrapped.borrow()),
        ]
    }
}

impl Unwrappable for Title {
    fn is_unwrapped(&self) -> bool {
        *self.unwrapped.borrow()
    }

    fn set_unwrapped(&mut self, unwrapped: bool) -> bool {
        let mut value = self.unwrapped.borrow_mut();
        *value = unwrapped;
        true
    }

    fn render_wrapped(&self) -> Html {
        html! {
            <h1 class={self.classes()}>
                { "Hello World!" }
            </h1>
        }
    }

    fn render_unwrapped(&self) -> Html {
        html! {
            <div>
                <h1 class={self.classes()}>
                    { "Hello Unwrapped World!" }
                </h1>
                <p class="text-lg mt-2 animate-pulse">
                    {"This is the unwrapped content with extra details!"}
                </p>
            </div>
        }
    }
}

impl UnwrappableComponent for Title {}

impl Component for Title {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Title::create_component()
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.render()
    }
}
