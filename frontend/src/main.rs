mod app;
use app::MainContent;

fn main() {
    leptos::mount::mount_to_body(MainContent);
}
