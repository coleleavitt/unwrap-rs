use yew::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

/// Represents a social media link with triple redundant properties
#[derive(Debug, Clone)]
struct SafeLink {
    urls: [String; 3],         // Triple redundant URL storage
    icon_paths: [String; 3],   // Triple redundant icon path storage
    labels: [String; 3],       // Triple redundant label storage
    active: [bool; 3],         // Link status with redundancy
}

impl SafeLink {
    /// Creates a new safety-hardened link with redundancy
    fn new(url: &str, icon_path: &str, label: &str) -> Self {
        Self {
            urls: [url.to_string(), url.to_string(), url.to_string()],
            icon_paths: [icon_path.to_string(), icon_path.to_string(), icon_path.to_string()],
            labels: [label.to_string(), label.to_string(), label.to_string()],
            active: [true, true, true],
        }
    }

    /// Gets the URL with error checking
    fn url(&self) -> String {
        let valid_urls: Vec<&String> = self.urls.iter()
            .filter(|u| !u.is_empty())
            .collect();

        if valid_urls.len() >= 2 && valid_urls[0] == valid_urls[1] {
            valid_urls[0].clone()
        } else if valid_urls.len() >= 3 && valid_urls[1] == valid_urls[2] {
            valid_urls[1].clone()
        } else if valid_urls.len() >= 3 && valid_urls[0] == valid_urls[2] {
            valid_urls[0].clone()
        } else if !valid_urls.is_empty() {
            // Fallback with warning
            log::warn!("URL redundancy check failed, using available URL");
            valid_urls[0].clone()
        } else {
            // Critical failure - use safe default
            log::error!("No valid URLs available, using empty URL");
            "#".to_string()
        }
    }

    /// Gets the icon path with error checking
    fn icon_path(&self) -> String {
        // Similar redundancy check as URL
        let valid_paths: Vec<&String> = self.icon_paths.iter()
            .filter(|p| !p.is_empty())
            .collect();

        if valid_paths.len() >= 2 && valid_paths[0] == valid_paths[1] {
            valid_paths[0].clone()
        } else if !valid_paths.is_empty() {
            valid_paths[0].clone()
        } else {
            // Fallback to generic icon
            "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/github/github-original.svg".to_string()
        }
    }

    /// Gets the label with error checking
    fn label(&self) -> String {
        // Similar redundancy check
        let valid_labels: Vec<&String> = self.labels.iter()
            .filter(|l| !l.is_empty())
            .collect();

        if valid_labels.len() >= 2 && valid_labels[0] == valid_labels[1] {
            valid_labels[0].clone()
        } else if !valid_labels.is_empty() {
            valid_labels[0].clone()
        } else {
            "Link".to_string()
        }
    }

    /// Checks if link is active (requires 2/3 agreement)
    fn is_active(&self) -> bool {
        self.active.iter().filter(|&&a| a).count() >= 2
    }
}

pub struct SocialLinks {
    links: Rc<RefCell<Vec<SafeLink>>>,
    hover_state: Rc<RefCell<Option<usize>>>,
}

impl Component for SocialLinks {
    type Message = SocialLinksMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Create hardened links with redundancy
        let links = vec![
            SafeLink::new(
                "https://github.com/coleleavitt",
                "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/github/github-original.svg",
                ""
            ),
            SafeLink::new(
                "https://www.linkedin.com/in/cole-leavitt/",
                "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/linkedin/linkedin-original.svg",
                ""
            ),
            SafeLink::new(
                "mailto:cole@unwrap.rs",
                "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.10.5/icons/envelope-fill.svg",
                ""
            ),
        ];

        // Schedule periodic verification
        let link = ctx.link().clone();
        let verify_interval = gloo_timers::callback::Interval::new(
            3000,
            move || link.send_message(SocialLinksMsg::VerifyLinks)
        );
        verify_interval.forget();

        SocialLinks {
            links: Rc::new(RefCell::new(links)),
            hover_state: Rc::new(RefCell::new(None)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SocialLinksMsg::SetHover(index) => {
                *self.hover_state.borrow_mut() = Some(index);
                true
            },
            SocialLinksMsg::ClearHover => {
                *self.hover_state.borrow_mut() = None;
                true
            },
            SocialLinksMsg::VerifyLinks => {
                // Periodic verification of link integrity
                let mut links = self.links.borrow_mut();
                for link in links.iter_mut() {
                    // Verify URL redundancy
                    if !link.urls.iter().all(|u| u == &link.urls[0]) {
                        // Self-heal by voting
                        let urls: Vec<&String> = link.urls.iter()
                            .filter(|u| !u.is_empty())
                            .collect();

                        if urls.len() >= 2 && urls[0] == urls[1] {
                            let correct_url = urls[0].clone();
                            link.urls = [correct_url.clone(), correct_url.clone(), correct_url];
                        }
                    }

                    // Similar checks could be done for other fields
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let links = self.links.borrow();
        let hover_state = *self.hover_state.borrow();

        let on_mouseenter = |index: usize| {
            ctx.link().callback(move |_| SocialLinksMsg::SetHover(index))
        };

        let on_mouseleave = ctx.link().callback(|_| SocialLinksMsg::ClearHover);

        html! {
            <section class="social-links">
                <div class="social-links-container">
                    {
                        links.iter().enumerate().map(|(index, link)| {
                            let is_hovered = hover_state == Some(index);
                            let classes = if is_hovered {
                                "social-link-item hovered"
                            } else {
                                "social-link-item"
                            };

                            if link.is_active() {
                                html! {
                                    <a
                                        href={link.url()}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class={classes}
                                        onmouseenter={on_mouseenter(index)}
                                        onmouseleave={on_mouseleave.clone()}
                                        data-label={link.label()}
                                    >
                                        <div class="icon-container">
                                            <img src={link.icon_path()} alt={link.label()} />
                                            <span class="link-label">{link.label()}</span>
                                        </div>
                                    </a>
                                }
                            } else {
                                html! {
                                    <div class="social-link-item disabled">
                                        <div class="icon-container">
                                            <img src={link.icon_path()} alt={link.label()} />
                                            <span class="link-label">{link.label()} {" (Unavailable)"}</span>
                                        </div>
                                    </div>
                                }
                            }
                        }).collect::<Html>()
                    }
                </div>
            </section>
        }
    }
}
#[derive(Debug, Clone)]
pub enum SocialLinksMsg {
    SetHover(usize),
    ClearHover,
    VerifyLinks,
}
