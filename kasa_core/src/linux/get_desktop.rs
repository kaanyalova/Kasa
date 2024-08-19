use std::env;

pub fn get_desktop_impl() -> Option<String> {
    if env::consts::OS == "linux" {
        match env::var("XDG_CURRENT_DESKTOP") {
            Ok(desktop) => return Some(desktop),
            Err(_) => return None,
        }
    } else {
        None
    }
}
