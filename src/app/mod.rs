mod social_links;
mod typing_animation;

pub use social_links::SocialLinks;
pub use typing_animation::TypingAnimation;

use yew::prelude::*;

#[derive(Debug)]
pub struct MainContent;

impl Component for MainContent {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        MainContent
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

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
