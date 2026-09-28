use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct AuthState {
    access_token: Signal<Option<String>>,
    // True until the initial attempt to restore a session has settled. Route
    // guards must wait for this instead of treating "no token yet" as
    // "logged out" on app start.
    //
    // On leitsys_web this is driven by a silent refresh against the
    // HttpOnly cookie. On mobile there is no such cookie — the refresh
    // token will instead live in the platform keychain (iOS) / keystore
    // (Android), which isn't wired up yet. Until then, `restoring` is
    // never resolved by anything and stays true unless a caller invokes
    // `finish_restoring` itself.
    restoring: Signal<bool>,
}

impl AuthState {
    pub fn login(&mut self, access_token: String) {
        self.access_token.set(Some(access_token));
    }

    pub fn logout(&mut self) {
        self.access_token.set(None);
    }

    pub fn is_logged_in(&self) -> bool {
        self.access_token.read().is_some()
    }

    pub fn is_restoring(&self) -> bool {
        *self.restoring.read()
    }

    pub fn finish_restoring(&mut self) {
        self.restoring.set(false);
    }

    pub fn token(&self) -> Option<String> {
        self.access_token.read().clone()
    }
}

pub fn provide_auth() -> AuthState {
    use_context_provider(|| AuthState {
        access_token: Signal::new(None),
        restoring: Signal::new(true),
    })
}

pub fn use_auth() -> AuthState {
    use_context::<AuthState>()
}
