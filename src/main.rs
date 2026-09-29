use std::time::Duration;

use dioxus::prelude::*;
use dioxus_sdk_time::sleep;

mod api;
mod auth;
mod components;
mod routes;
mod views;

use auth::provide_auth;
use components::toast::ToastProvider;
use routes::Route;

/// The access token expires after 15 minutes (see `generate_access_token`
/// server-side); refresh well before that so the app left open doesn't
/// start hitting 401s mid-session.
const REFRESH_INTERVAL: Duration = Duration::from_secs(12 * 60);

const FAVICON: Asset = asset!("/assets/favicon.ico");
const DX_COMPONENTS_CSS: Asset = asset!("/assets/dx-components-theme.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut auth = provide_auth();

    use_effect(move || {
        spawn(async move {
            if let Ok(token) = api::refresh().await {
                auth.login(token);
            }
            auth.finish_restoring();

            loop {
                sleep(REFRESH_INTERVAL).await;
                if auth.is_logged_in() {
                    if let Ok(token) = api::refresh().await {
                        auth.login(token);
                    }
                }
            }
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: DX_COMPONENTS_CSS }
        ToastProvider {
            Router::<Route> {}
        }
    }
}
