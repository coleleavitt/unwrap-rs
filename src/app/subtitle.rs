use super::{YewComponent, StyledComponent, Unwrappable, UnwrappableComponent};
use yew::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Subtitle {
    unwrapped: Rc<RefCell<bool>>,
}

impl YewComponent for Subtitle {
    fn render(&self) -> Html {
        self.render_with_unwrap_state()
    }

    fn create_component() -> Self {
        Subtitle {
            unwrapped: Rc::new(RefCell::new(false)),
        }
    }
}

impl StyledComponent for Subtitle {
    fn base_classes(&self) -> Vec<String> {
        vec!["block".into(), "-mt-4".into()]
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

impl Unwrappable for Subtitle {
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
            <span class={self.classes()}>
                { "from Yew with " }
                <i class="heart" />
            </span>
        }
    }

    fn render_unwrapped(&self) -> Html {
        html! {
            <div>
                <span class={self.classes()}>
                    { "Unwrapped Subtitle Content!" }
                </span>
                <p class="text-lg mt-2 animate-pulse">
                    {"Additional details for the unwrapped subtitle."}
                </p>
            </div>
        }
    }
}

impl UnwrappableComponent for Subtitle {}

impl Component for Subtitle {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Subtitle::create_component()
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.render()
    }
}
