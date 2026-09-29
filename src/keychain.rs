use keyring::Entry;

/// iOS Keychain service/account pair the refresh-token cookie is stored
/// under, so a session survives an app restart instead of always starting
/// logged out. Android keystore isn't wired up yet — `keyring`'s Android
/// backend needs the `cli` feature and hasn't been tested on this project.
const SERVICE: &str = "com.leitsys.mobile";
const ACCOUNT: &str = "refresh_token";

pub fn store_refresh_token(token: &str) {
    if let Ok(entry) = Entry::new(SERVICE, ACCOUNT) {
        let _ = entry.set_password(token);
    }
}

/// Returns `None` both when nothing has been stored yet and when the
/// platform keychain is unavailable — either way there's no session to
/// restore, which the caller already handles the same way.
pub fn load_refresh_token() -> Option<String> {
    Entry::new(SERVICE, ACCOUNT).ok()?.get_password().ok()
}

pub fn clear_refresh_token() {
    if let Ok(entry) = Entry::new(SERVICE, ACCOUNT) {
        let _ = entry.delete_credential();
    }
}
