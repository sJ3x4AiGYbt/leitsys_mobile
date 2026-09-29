use dioxus::prelude::*;
use crate::api;
use crate::auth::use_auth;
use crate::keychain;
use crate::routes::Route;

/// Placeholder landing page after login — the real dashboard (categories,
/// due questions, calendar) from leitsys_web/src/views/home.rs isn't ported
/// yet. This exists so routes.rs has a real destination to navigate/guard.
#[component]
pub fn Home() -> Element {
    let mut auth = use_auth();
    let nav = use_navigator();

    let username = auth.token().and_then(|token| api::decode_claims(&token)).map(|claims| claims.username);
    let greeting = match username {
        Some(username) => format!("Welcome, {username}!"),
        None => "Welcome!".to_string(),
    };

    let on_logout = move |_| {
        spawn(async move {
            let refresh_token = keychain::load_refresh_token();
            let _ = api::logout(refresh_token.as_deref()).await;
            keychain::clear_refresh_token();
            auth.logout();
            nav.replace(Route::Login {});
        });
    };

    rsx! {
        div {
            style: "max-width: 400px; margin: 5rem auto; text-align: center; display: flex; flex-direction: column; gap: 1rem;",
            h1 { "{greeting}" }
            p { "This is a placeholder — the real dashboard isn't ported yet." }
            button { onclick: on_logout, "Log out" }
        }
    }
}
