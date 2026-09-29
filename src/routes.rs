use dioxus::prelude::*;
use crate::auth::use_auth;
use crate::views::{Login, Home};

#[rustfmt::skip]
#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")]
    Root {},
    #[route("/login")]
    Login {},
    #[layout(RequireAuth)]
        #[route("/home")]
        Home {},
}

#[component]
fn Root() -> Element {
    let auth = use_auth();
    let nav = use_navigator();

    use_effect(move || {
        // Wait for the silent-refresh attempt to settle — on app start the
        // token is still empty regardless of whether a restored session
        // will land a moment later.
        if auth.is_restoring() {
            return;
        }
        if auth.is_logged_in() {
            nav.replace(Route::Home {});
        } else {
            nav.replace(Route::Login {});
        }
    });

    rsx! { div {} }
}

/// Layout guarding the authenticated routes — redirects to `/login` if the
/// user has no access token instead of rendering the nested route.
#[component]
fn RequireAuth() -> Element {
    let auth = use_auth();
    let nav = use_navigator();

    use_effect(move || {
        if !auth.is_restoring() && !auth.is_logged_in() {
            nav.replace(Route::Login {});
        }
    });

    if auth.is_restoring() {
        // Avoid flashing the protected page's content before we know
        // whether the restored session actually holds.
        rsx! { div {} }
    } else if auth.is_logged_in() {
        rsx! { Outlet::<Route> {} }
    } else {
        rsx! { div {} }
    }
}
