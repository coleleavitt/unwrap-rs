mod app;

use wasm_bindgen::JsCast;

fn main() {
    let window = web_sys::window().expect("the app only runs in a browser window");

    // The static `.unwrap()` in index.html stays when motion is unwanted.
    let reduce_motion = window
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .is_some_and(|query| query.matches());
    if reduce_motion {
        return;
    }

    let mount_point = window
        .document()
        .and_then(|doc| doc.get_element_by_id("animation-mount"))
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
        .expect("index.html provides the #animation-mount element");

    leptos::mount::mount_to(mount_point, app::TypingAnimation).forget();
}
