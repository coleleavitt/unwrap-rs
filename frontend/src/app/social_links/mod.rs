use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq)]
struct SocialLink {
    url: &'static str,
    icon_path: &'static str,
    label: &'static str,
}

const SOCIAL_LINKS: &[SocialLink] = &[
    SocialLink {
        url: "https://github.com/coleleavitt",
        icon_path: "icons/github-original.svg",
        label: "GitHub",
    },
    SocialLink {
        url: "https://www.linkedin.com/in/coleleavitt/",
        icon_path: "icons/linkedin.svg",
        label: "LinkedIn",
    },
    SocialLink {
        url: "mailto:cole@unwrap.rs",
        icon_path: "icons/envelope-fill.svg",
        label: "Email",
    },
];

#[component]
pub fn SocialLinks() -> impl IntoView {
    let hover_index = RwSignal::new(None::<usize>);

    view! {
        <section class="social-links" aria-label="Connect with Cole Leavitt">
            <div class="social-links-container">
                {SOCIAL_LINKS
                    .iter()
                    .enumerate()
                    .map(|(index, link)| {
                        let classes = move || {
                            if hover_index.get() == Some(index) {
                                "social-link-item hovered"
                            } else {
                                "social-link-item"
                            }
                        };
                        view! {
                            <a
                                href=link.url
                                target="_blank"
                                rel="noopener noreferrer"
                                class=classes
                                on:mouseenter=move |_| hover_index.set(Some(index))
                                on:mouseleave=move |_| hover_index.set(None)
                                data-label=link.label
                            >
                                <div class="icon-container">
                                    <img src=link.icon_path alt=link.label />
                                    <span class="link-label">{link.label}</span>
                                </div>
                            </a>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}
