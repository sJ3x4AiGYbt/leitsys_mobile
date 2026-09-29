use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use crate::api;
use crate::auth::use_auth;
use crate::routes::Route;
use crate::components::card::{Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter};
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::password_rules::PasswordRules;

#[derive(Clone, Copy, PartialEq)]
enum AuthView {
    Login,
    Signup,
    ForgotPassword,
    ResendVerification,
}

#[component]
pub fn Login() -> Element {
    let view = use_signal(|| AuthView::Login);

    rsx! {
        div {
            style: "max-width: 400px; margin: 5rem auto;",
            Card {
                style: "width: 100%; max-width: 24rem;",
                match view() {
                    AuthView::Login => rsx! { LoginForm { view } },
                    AuthView::Signup => rsx! { SignupForm { view } },
                    AuthView::ForgotPassword => rsx! { ForgotPasswordForm { view } },
                    AuthView::ResendVerification => rsx! { ResendVerificationForm { view } },
                }
            }
        }
    }
}


#[component]
fn LoginForm(mut view: Signal<AuthView>) -> Element {
    let mut auth = use_auth();
    let nav = use_navigator();
    let toast = use_toast();

    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut submitting = use_signal(|| false);

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        if submitting() {
            return;
        }
        submitting.set(true);

        spawn(async move {
            match api::login(&username(), &password()).await {
                Ok(token) => {
                    auth.login(token);
                    nav.push(Route::Home {});
                }
                Err(message) => {
                    let needs_verification = message.contains("verify your email");
                    toast.error(message, ToastOptions::new());
                    submitting.set(false);
                    if needs_verification {
                        view.set(AuthView::ResendVerification);
                    }
                }
            }
        });
    };

    rsx! {
        CardHeader {
            CardTitle { "Log in to your account" }
            CardDescription { "Enter your username below to login to your account" }
        }
        CardContent {
            form {
                id: "login-form",
                onsubmit: on_submit,
                div { style: "display: flex; flex-direction: column; gap: 1.5rem;",
                    div { style: "display: grid; gap: 0.5rem;",
                        Label { html_for: "username", "Username" }
                        Input {
                            id: "username",
                            r#type: "text",
                            placeholder: "jane.doe",
                            value: "{username}",
                            oninput: move |e: FormEvent| username.set(e.value()),
                        }
                    }
                    div { style: "display: grid; gap: 0.5rem;",
                        div { style: "display: flex; align-items: center;",
                            Label { html_for: "password", "Password" }
                            a {
                                onclick: move |_| view.set(AuthView::ForgotPassword),
                                style: "margin-left: auto; font-size: 0.875rem; color: var(--secondary-color-5); text-decoration: underline; text-underline-offset: 4px;",
                                "Forgot your password?"
                            }
                        }
                        Input {
                            id: "password",
                            r#type: "password",
                            value: "{password}",
                            oninput: move |e: FormEvent| password.set(e.value()),
                        }
                    }
                }
            }
        }
        CardFooter { style: "flex-direction: column; gap: 0.5rem;",
            Button {
                r#type: "submit",
                form: "login-form",
                style: "width: 100%;",
                disabled: submitting(),
                if submitting() { "Logging in..." } else { "Login" }
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| view.set(AuthView::Signup),
                style: "width: 100%;",
                "Sign Up"
            }
            a {
                onclick: move |_| view.set(AuthView::ResendVerification),
                style: "font-size: 0.875rem; color: var(--secondary-color-5); text-decoration: underline; text-underline-offset: 4px; cursor: pointer;",
                "Didn't get a verification email?"
            }
        }
    }
}

#[component]
fn SignupForm(mut view: Signal<AuthView>) -> Element {
    let toast = use_toast();

    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut submitting = use_signal(|| false);

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        if submitting() {
            return;
        }
        submitting.set(true);

        spawn(async move {
            match api::register(&username(), &email(), &password()).await {
                Ok(message) => {
                    submitting.set(false);
                    toast.success(message, ToastOptions::new());
                    view.set(AuthView::Login);
                }
                Err(message) => {
                    toast.error(message, ToastOptions::new());
                    submitting.set(false);
                }
            }
        });
    };

    rsx! {
        CardHeader {
            CardTitle { "Create an account" }
            CardDescription { "Fill in your information to get started" }
        }
        CardContent {
            form {
                id: "signup-form",
                onsubmit: on_submit,
                div { style: "display: flex; flex-direction: column; gap: 1.5rem;",
                    div { style: "display: grid; gap: 0.5rem;",
                        Label { html_for: "username", "Username" }
                        Input {
                            id: "username",
                            r#type: "text",
                            placeholder: "jane.doe",
                            value: "{username}",
                            oninput: move |e: FormEvent| username.set(e.value()),
                        }
                    }
                    div { style: "display: grid; gap: 0.5rem;",
                        Label { html_for: "email", "Email" }
                        Input {
                            id: "email",
                            r#type: "email",
                            placeholder: "m@example.com",
                            value: "{email}",
                            oninput: move |e: FormEvent| email.set(e.value()),
                        }
                    }
                    div { style: "display: grid; gap: 0.5rem;",
                        Label { html_for: "password", "Password" }
                        Input {
                            id: "password",
                            r#type: "password",
                            value: "{password}",
                            oninput: move |e: FormEvent| password.set(e.value()),
                        }
                        PasswordRules { password: password() }
                    }
                }
            }
        }
        CardFooter { style: "flex-direction: column; gap: 0.5rem;",
            Button {
                r#type: "submit",
                form: "signup-form",
                style: "width: 100%;",
                disabled: submitting(),
                if submitting() { "Creating account..." } else { "Sign up" }
            }
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| view.set(AuthView::Login),
                style: "width: 100%;",
                "Log in"
            }
        }
    }
}

#[component]
fn ForgotPasswordForm(mut view: Signal<AuthView>) -> Element {
    let toast = use_toast();

    let mut email = use_signal(String::new);
    let mut submitting = use_signal(|| false);

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        if submitting() {
            return;
        }
        submitting.set(true);

        spawn(async move {
            match api::forgot_password(&email()).await {
                Ok(message) => {
                    submitting.set(false);
                    toast.success(message, ToastOptions::new());
                    view.set(AuthView::Login);
                }
                Err(message) => {
                    toast.error(message, ToastOptions::new());
                    submitting.set(false);
                }
            }
        });
    };

    rsx! {
        CardHeader {
            CardTitle { "Forgot password" }
            CardDescription { "Enter your email and we'll send you a reset link" }
        }
        CardContent {
            form {
                id: "forgot-password-form",
                onsubmit: on_submit,
                div { style: "display: grid; gap: 0.5rem;",
                    Label { html_for: "email", "Email" }
                    Input {
                        id: "email",
                        r#type: "email",
                        placeholder: "m@example.com",
                        value: "{email}",
                        oninput: move |e: FormEvent| email.set(e.value()),
                    }
                }
            }
        }
        CardFooter { style: "flex-direction: column; gap: 0.5rem;",
            Button {
                r#type: "submit",
                form: "forgot-password-form",
                style: "width: 100%;",
                disabled: submitting(),
                if submitting() { "Sending..." } else { "Send link" }
            }
            Button {
                variant: ButtonVariant::Ghost,
                style: "width: 100%;",
                onclick: move |_| view.set(AuthView::Login),
                "Back to login"
            }
        }
    }
}

#[component]
fn ResendVerificationForm(mut view: Signal<AuthView>) -> Element {
    let toast = use_toast();

    let mut email = use_signal(String::new);
    let mut submitting = use_signal(|| false);

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        if submitting() {
            return;
        }
        submitting.set(true);

        spawn(async move {
            match api::resend_verification(&email()).await {
                Ok(message) => {
                    submitting.set(false);
                    toast.success(message, ToastOptions::new());
                    view.set(AuthView::Login);
                }
                Err(message) => {
                    toast.error(message, ToastOptions::new());
                    submitting.set(false);
                }
            }
        });
    };

    rsx! {
        CardHeader {
            CardTitle { "Resend verification email" }
            CardDescription { "Enter your email and we'll send you a new verification link" }
        }
        CardContent {
            form {
                id: "resend-verification-form",
                onsubmit: on_submit,
                div { style: "display: grid; gap: 0.5rem;",
                    Label { html_for: "email", "Email" }
                    Input {
                        id: "email",
                        r#type: "email",
                        placeholder: "m@example.com",
                        value: "{email}",
                        oninput: move |e: FormEvent| email.set(e.value()),
                    }
                }
            }
        }
        CardFooter { style: "flex-direction: column; gap: 0.5rem;",
            Button {
                r#type: "submit",
                form: "resend-verification-form",
                style: "width: 100%;",
                disabled: submitting(),
                if submitting() { "Sending..." } else { "Send link" }
            }
            Button {
                variant: ButtonVariant::Ghost,
                style: "width: 100%;",
                onclick: move |_| view.set(AuthView::Login),
                "Back to login"
            }
        }
    }
}
