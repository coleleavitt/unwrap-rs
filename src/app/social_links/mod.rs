use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
struct SocialLink {
    url: String,
    icon_path: String,
    label: String,
}

impl SocialLink {
    fn new(url: &str, icon_path: &str, label: &str) -> Self {
        Self {
            url: url.to_string(),
            icon_path: icon_path.to_string(),
            label: label.to_string(),
        }
    }
}

pub struct SocialLinks {
    links: Vec<SocialLink>,
    hover_index: Option<usize>,
}

pub enum SocialLinksMsg {
    SetHover(usize),
    ClearHover,
}

impl Component for SocialLinks {
    type Message = SocialLinksMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let links = vec![
            SocialLink::new(
                "https://github.com/coleleavitt",
                "icons/github-original.svg",
                "GitHub",
            ),
            SocialLink::new(
                "https://www.linkedin.com/in/coleleavitt/",
                "icons/linkedin.svg",
                "LinkedIn",
            ),
            SocialLink::new("mailto:cole@unwrap.rs", "icons/envelope-fill.svg", "Email"),
        ];

        SocialLinks {
            links,
            hover_index: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SocialLinksMsg::SetHover(index) => {
                self.hover_index = Some(index);
                true
            }
            SocialLinksMsg::ClearHover => {
                self.hover_index = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_mouseleave = ctx.link().callback(|_| SocialLinksMsg::ClearHover);

        html! {
            <section class="social-links">
                <div class="social-links-container">
                    {
                        self.links.iter().enumerate().map(|(index, link)| {
                            let is_hovered = self.hover_index == Some(index);
                            let classes = if is_hovered {
                                "social-link-item hovered"
                            } else {
                                "social-link-item"
                            };

                            let on_mouseenter = ctx.link()
                                .callback(move |_| SocialLinksMsg::SetHover(index));

                            html! {
                                <a
                                    href={link.url.clone()}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class={classes}
                                    onmouseenter={on_mouseenter}
                                    onmouseleave={on_mouseleave.clone()}
                                    data-label={link.label.clone()}
                                >
                                    <div class="icon-container">
                                        <img src={link.icon_path.clone()} alt={link.label.clone()} />
                                        <span class="link-label">{&link.label}</span>
                                    </div>
                                </a>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </section>
        }
    }
}
