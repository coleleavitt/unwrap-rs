// src/app/mod.rs
mod typing_animation; // Declare the module
mod social_links;

pub use typing_animation::TypingAnimation; // Make TypingAnimation public
pub use social_links::SocialLinks;



use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

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
            // Use the CSS class for the main container
            <main class="radiation-hardened-container">
                // Container specifically for the text and animation area
                <div class="transformium-container">
                    // Static text part
                    <span class="base-text">{"Result<T, E>"}</span> // Updated base text example
                    // The animation component itself
                    <TypingAnimation />
                    // Optional: Blinking cursor effect after the animation
                    // <span class="cursor-blink">{"_"}</span>
                </div>
            <SocialLinks />
            </main>
        }
    }
}
