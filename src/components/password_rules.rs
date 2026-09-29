use dioxus::prelude::*;

/// Mirrors the backend's `validate_password` (models.rs): 10-72 characters,
/// at least one lowercase letter, one uppercase letter, one digit and one
/// special character from this exact set.
const SPECIAL_CHARS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?/~`\"'\\";

#[component]
pub fn PasswordRules(password: String) -> Element {
    let long_enough = password.chars().count() >= 10 && password.len() <= 72;
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| SPECIAL_CHARS.contains(c));

    rsx! {
        ul {
            style: "margin: 0; padding-left: 1.1rem; font-size: 0.8rem; display: flex; flex-direction: column; gap: 0.15rem;",
            PasswordRule { met: long_enough, label: "10-72 characters" }
            PasswordRule { met: has_lower, label: "one lowercase letter" }
            PasswordRule { met: has_upper, label: "one uppercase letter" }
            PasswordRule { met: has_digit, label: "one digit" }
            PasswordRule { met: has_special, label: "one special character (e.g. !@#$%)" }
        }
    }
}

#[component]
fn PasswordRule(met: bool, label: &'static str) -> Element {
    let color = if met { "#2ecc71" } else { "#888" };
    rsx! {
        li { style: "color: {color};", if met { "✓ " } else { "○ " } "{label}" }
    }
}
