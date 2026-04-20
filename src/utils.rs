use base64::Engine;

pub fn random_b64url(len: usize) -> String {
    let mut buf = vec![0u8; len];
    getrandom::fill(&mut buf).unwrap();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf)
}

pub fn get_window() -> web_sys::Window {
    web_sys::window().unwrap() // unwrap is ok because we are in the browser
}

pub fn get_location() -> web_sys::Location {
    get_window().location()
}

pub fn get_local_storage() -> web_sys::Storage {
    get_window().local_storage().unwrap().unwrap() // unwrap is ok because we are in the browser
}
