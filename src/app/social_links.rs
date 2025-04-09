use yew::prelude::*;

pub struct SocialLinks;


impl Component for SocialLinks {
    type Message = ();
    type Properties = ();


    fn create(_ctx: &Context<Self>) -> Self {
        SocialLinks
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <section class ="social-links">
                <a href="https://github.com/coleleavitt" target="_blank" rel="noopener noreferrer">
                    <img src="https://cdn.jsdelivr.net/gh/devicons/devicon/icons/github/github-original.svg" alt="GitHub" />
                </a>
                <a href="https://www.linkedin.com/in/cole-leavitt/" target="_blank" rel="noopener noreferrer">
                    <img src="https://cdn.jsdelivr.net/gh/devicons/devicon/icons/linkedin/linkedin-original.svg" alt="LinkedIn" />
                </a>
                <a href="mailto:cole@unwrap.rs" target="_blank" rel="noopener noreferrer">
                    <
            </a>
            </section>
        }
    }
}