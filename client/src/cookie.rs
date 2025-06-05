use crate::*;

pub fn acquire_cookie() -> String {
    let cookie_options = cookies::CookieOptions::default()
        .expires_after(core::time::Duration::from_secs(52 * 7 * 24 * 60 * 60));
    match cookies::get("test") {
        Some(Ok(cookie)) => {
            // log!("got cookie");
            return cookie;
        }
        Some(Err(_)) => {
            // log!(format!("cookie error: {}", e));
        }
        None => {
            // log!("did not find cookie");
        }
    }
    // log!("setting cookie");
    cookies::set("test", "123", &cookie_options);
    "123".to_string()
}

pub fn render_cookie(cookie: &str) -> Html {
    html! {
        <div>
            <p>{cookie}</p>
        </div>
    }
}
