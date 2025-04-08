// src/main.rs
mod app;
use app::MainContent;

fn main() {
    yew::Renderer::<MainContent>::new().render();
}
