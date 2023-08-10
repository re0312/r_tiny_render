pub mod math;
pub mod color;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let dst = document.get_element_by_id("wasm-example").unwrap();
    let canvas = document.create_element("canvas").unwrap();
    dst.append_child(&canvas);
    // Manufacture the element we're gonna append
}
