/// Scroll the element with the given id into the centre of the viewport.
///
/// Waits a short tick so the DOM has a chance to render the target, then
/// applies a temporary `search-highlight` class that fades the navbar-coloured
/// red tint over 3 seconds.
/// This is a no-op on the server (non-hydrate builds).
pub fn scroll_to_id(_id: &str) {
    #[cfg(feature = "hydrate")]
    {
        use leptos::task;
        use wasm_bindgen::JsCast;
        let id = _id.to_string();
        task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(50).await;
            let window = web_sys::window();
            let document = window.as_ref().and_then(|w| w.document());
            let element = document.as_ref().and_then(|d| d.get_element_by_id(&id));
            if let Some(element) = element {
                let _ = element.class_list().remove_1("search-highlight");
                let _ = element.class_list().add_1("search-highlight");
                let el: js_sys::Object = element.unchecked_into();
                let opts = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&opts, &"behavior".into(), &"smooth".into());
                let _ = js_sys::Reflect::set(&opts, &"block".into(), &"center".into());
                let _ = js_sys::Reflect::set(&opts, &"inline".into(), &"center".into());
                if let Ok(scroll_into_view) = js_sys::Reflect::get(&el, &"scrollIntoView".into()) {
                    if let Ok(func) = scroll_into_view.dyn_into::<js_sys::Function>() {
                        let _ = func.call1(&el, &opts);
                    }
                }

                let id_for_remove = id.clone();
                task::spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(3000).await;
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Some(element) = document.get_element_by_id(&id_for_remove) {
                                let _ = element.class_list().remove_1("search-highlight");
                            }
                        }
                    }
                });
            }
        });
    }
}
