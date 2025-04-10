// src/app/mod.rs
mod social_links;
mod typing_animation; // Declare the module

pub use social_links::SocialLinks;
pub use typing_animation::TypingAnimation; // Make TypingAnimation public

use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;

/// Trait defining core component behavior (ensure this matches your actual trait if different)
/// If this trait isn't actually used by TypingAnimation directly, it can be removed.
pub trait YewComponent: Component {
    /// Renders the component with bounded execution time
    fn render(&self) -> Html;

    /// Creates a new component instance with static memory allocation
    fn create_component() -> Self;
}

/// Main application component
#[derive(Debug)] // Add Debug derive
pub struct MainContent {
    #[allow(dead_code)]
    animation_state: Rc<RefCell<bool>>, // Example state, currently unused
}

impl Component for MainContent {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        MainContent {
            animation_state: Rc::new(RefCell::new(false)),
        }
    }

    // Removed update method as there are no messages

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <main class="radiation-hardened-container">
                <div class="transformium-container">
                    <span class="base-text">{"Result<T, E>"}</span><TypingAnimation />
                </div>
                <SocialLinks />
            </main>
        }
    }
}
