use js_sys::Uint8Array;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Blob;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    Ok(())
}

#[wasm_bindgen(getter_with_clone)]
pub struct PDFOutput {
    pub data: web_sys::Blob,
    pub name: String,
}

#[wasm_bindgen]
pub async fn remove_pages(file_id: &str, text_id: &str) -> PDFOutput {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    // log(&format!(
    //     "{:?}",
    //     document
    //         .get_element_by_id(file_id)
    //         .unwrap()
    //         .value_of()
    //         .dyn_into::<web_sys::HtmlInputElement>()
    // ));

    let file = document
        .get_element_by_id(file_id)
        .unwrap()
        .value_of()
        .dyn_into::<web_sys::HtmlInputElement>()
        .unwrap();
    let text = document
        .get_element_by_id(text_id)
        .unwrap()
        .value_of()
        .dyn_into::<web_sys::HtmlInputElement>()
        .unwrap();

    let file = file.files().unwrap();

    // log(&format!("{:?}", file));

    let file = file.get(0).unwrap();

    let filename = std::path::PathBuf::from(file.name());

    let mut newname = filename
        .file_stem()
        .and_then(|v| v.to_str())
        .map(|v| v.to_string())
        .unwrap_or_else(|| "new".to_string());
    newname.push_str(".modified.pdf");

    let file_buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
        .await
        .unwrap()
        .dyn_into::<js_sys::ArrayBuffer>()
        .unwrap();
    let data: Vec<u8> = Uint8Array::new(&file_buffer).to_vec();

    let mut pdf = lopdf::Document::load_mem(&data).unwrap();

    let text = text.value();

    let page_numbers: Vec<u32> = text
        .split(',')
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(|v| v.parse().unwrap())
        .collect();

    log(&format!("{text}, {page_numbers:?}"));

    pdf.delete_pages(&page_numbers);

    let mut buffer = Vec::new();
    pdf.save_to(&mut buffer).unwrap();

    let output_buffer = Uint8Array::new_with_length(buffer.len() as u32);
    for (index, b) in buffer.into_iter().enumerate() {
        output_buffer.set_index(index as u32, b);
    }

    let array = js_sys::Array::new();
    array.push(&output_buffer.buffer());

    PDFOutput {
        data: Blob::new_with_u8_array_sequence(&array).unwrap(),
        name: newname,
    }
}
