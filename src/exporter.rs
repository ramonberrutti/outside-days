use web_sys::{Blob, Url, HtmlAnchorElement, HtmlInputElement, FileReader};
use web_sys::wasm_bindgen::JsCast;
use crate::trip::Trip;
use js_sys::Array;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;

pub fn export_download(trips: &[Trip]) {
    let csv = Trip::export_csv(trips);
    let blob = Blob::new_with_str_sequence(
        &Array::of1(&JsValue::from(csv.as_str()))
    ).expect("create blob");
    let url = Url::create_object_url_with_blob(&blob).expect("create object URL");
    let window = web_sys::window().expect("window");
    let document = window.document().expect("document");
    let a: HtmlAnchorElement = document
        .create_element("a")
        .expect("create anchor")
        .dyn_into()
        .expect("anchor");
    a.set_attribute("href", &url).expect("set href");
    a.set_attribute("download", "outside-days.csv").expect("set download");
    a.click();
    Url::revoke_object_url(&url).ok();
}

pub fn import_file_picker(on_import: impl Fn(Result<Vec<Trip>, String>) + 'static) {
    let window = web_sys::window().expect("window");
    let document = window.document().expect("document");
    let input: HtmlInputElement = document
        .create_element("input")
        .expect("create input")
        .dyn_into()
        .expect("input");
    input.set_type("file");
    input.set_attribute("accept", ".csv").ok();

    let input_clone = input.clone();
    let onchange = Closure::once(Box::new(move |_: web_sys::Event| {
        let files = input_clone.files();
        let file = files.and_then(|f| f.get(0));
        if let Some(file) = file {
            let reader = FileReader::new().expect("FileReader");
            let reader_clone = reader.clone();
            let onload = Closure::once(Box::new(move |_: web_sys::Event| {
                let result = reader_clone
                    .result()
                    .ok()
                    .and_then(|v| v.as_string());
                match result {
                    Some(csv) => on_import(Trip::import_csv(&csv)),
                    None => on_import(Err("Failed to read file".to_string())),
                }
            }));
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();
            reader.read_as_text(&file).ok();
        }
    }));
    input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
    onchange.forget();
    input.click();
}