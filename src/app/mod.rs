mod social_links;
mod typing_animation;

pub use social_links::SocialLinks;
pub use typing_animation::TypingAnimation;

use leptos::prelude::*;

#[component]
pub fn MainContent() -> impl IntoView {
    view! {
        <main class="radiation-hardened-container">
            <div class="transformium-container">
                <span class="base-text">"Result<T, E>"</span>
                <TypingAnimation />
            </div>
            <SocialLinks />
        </main>
    }
}
