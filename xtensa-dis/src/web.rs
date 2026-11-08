//! The live demo: every `.xtensa-demo` on the page re-disassembles its
//! bytes as you type. The page's HTML already holds a listing rendered at
//! build time, so without WebAssembly the demo still shows its example.

use wasm_bindgen::JsCast as _;
use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlInputElement, HtmlTextAreaElement};

use crate::{listing_html, parse_address, parse_hex};

fn update(bytes: &HtmlTextAreaElement, base: &HtmlInputElement, output: &Element) {
    match (parse_hex(&bytes.value()), parse_address(&base.value())) {
        // listing_html escapes everything it prints
        (Ok(code), Ok(address)) => output.set_inner_html(&listing_html(&code, address)),
        // errors quote the reader's input, so they go in as text only
        (Err(e), _) | (_, Err(e)) => output.set_text_content(Some(&format!("error: {e}"))),
    }
}

fn wire(demo: &Element) -> Option<()> {
    let bytes: HtmlTextAreaElement = demo
        .query_selector(".xtensa-bytes")
        .ok()??
        .dyn_into()
        .ok()?;
    let base: HtmlInputElement = demo.query_selector(".xtensa-base").ok()??.dyn_into().ok()?;
    let output = demo.query_selector(".xtensa-out").ok()??;
    let on_input = {
        let (bytes, base) = (bytes.clone(), base.clone());
        Closure::<dyn Fn()>::new(move || update(&bytes, &base, &output))
    };
    for target in [
        bytes.unchecked_ref::<web_sys::EventTarget>(),
        base.unchecked_ref(),
    ] {
        target
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .ok()?;
    }
    // the listeners live as long as the page
    on_input.forget();
    // sitegen renders the inputs read-only, for pages without WebAssembly
    bytes.set_read_only(false);
    base.set_read_only(false);
    demo.set_attribute("data-live", "").ok()?;
    Some(())
}

#[wasm_bindgen(start)]
pub fn start() {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(demos) = document.query_selector_all(".xtensa-demo") else {
        return;
    };
    for i in 0..demos.length() {
        if let Some(demo) = demos.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
            let _ = wire(&demo);
        }
    }
}
