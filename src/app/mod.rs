mod typing_animation;

pub use typing_animation::TypingAnimation;

use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

/// Trait defining core component behavior
pub trait YewComponent: Component {
    fn render(&self) -> Html;
    fn create_component() -> Self;
}

/// Main application component
pub struct MainContent {
    #[allow(dead_code)]
    animation_state: Rc<RefCell<bool>>,
}

impl Component for MainContent {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        MainContent {
            animation_state: Rc::new(RefCell::new(false)),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <main class="radiation-hardened-container">
                <TypingAnimation />
            </main>
        }
    }
}
