use super::{YewComponent, StyledComponent};
use yew::prelude::*;

pub struct Logo;

impl YewComponent for Logo {
    fn render(&self) -> Html {
        html! {
            <img class={self.classes()}
                 src="https://yew.rs/img/logo.svg"
                 alt="Yew logo" />
        }
    }

    fn create_component() -> Self {
        Logo
    }
}

impl StyledComponent for Logo {
    fn base_classes(&self) -> Vec<String> {
        vec!["h-80".into(), "mx-auto".into()]
    }
}

impl Component for Logo {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Logo
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.render()
    }
}
