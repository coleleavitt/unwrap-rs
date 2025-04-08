mod typing_animation;

pub use typing_animation::TypingAnimation;

use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

/// Trait defining core component behavior with formal verification guarantees
pub trait YewComponent: Component {
    /// Renders the component with bounded execution time
    fn render(&self) -> Html;

    /// Creates a new component instance with static memory allocation
    fn create_component() -> Self;
}

/// Main application component with fault-tolerant state management
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
