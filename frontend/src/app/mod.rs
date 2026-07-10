mod social_links;
mod typing_animation;

pub use social_links::SocialLinks;
pub use typing_animation::TypingAnimation;

use leptos::prelude::*;

#[component]
pub fn MainContent() -> impl IntoView {
    view! {
        <main class="radiation-hardened-container" aria-labelledby="portfolio-title">
            <header class="identity-lockup">
                <h1 id="portfolio-title" class="identity-name">"Cole Leavitt"</h1>
                <p class="identity-statement">
                    "I build and break systems software in "
                    <span class="identity-accent">"Rust"</span>
                    "."
                </p>
            </header>
            <div class="transformium-container" aria-label="Result of T, E dot unwrap">
                <span class="base-text" aria-hidden="true">"Result<T, E>"</span>
                <TypingAnimation />
            </div>
            <SocialLinks />
        </main>
    }
}
