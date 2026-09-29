use dioxus::prelude::*;
use dioxus_primitives::toast::{
    self, ToastCloseButtonProps, ToastContentProps, ToastDescriptionProps, ToastProps,
    ToastPropsWithOwner, ToastTitleProps,
};
use std::time::Duration;

#[component]
fn StyledToast(props: ToastProps) -> Element {
    rsx! {
        toast::Toast {
            id: props.id,
            index: props.index,
            title: props.title,
            description: props.description,
            toast_type: props.toast_type,
            on_close: props.on_close,
            permanent: props.permanent,
            duration: props.duration,
            class: "toast",
            attributes: props.attributes,
            ToastContent {
                ToastTitle {}
                ToastDescription {}
            }
            ToastCloseButton {}
        }
    }
}

#[component]
fn ToastContent(props: ToastContentProps) -> Element {
    rsx! {
        toast::ToastContent {
            class: "toast-content",
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
fn ToastTitle(props: ToastTitleProps) -> Element {
    rsx! {
        toast::ToastTitle {
            class: "toast-title",
            attributes: props.attributes,
            children: props.children,
        }
    }
}

#[component]
fn ToastDescription(props: ToastDescriptionProps) -> Element {
    rsx! {
        toast::ToastDescription {
            class: "toast-description",
            attributes: props.attributes,
            children: props.children,
        }
    }
}

#[component]
fn ToastCloseButton(props: ToastCloseButtonProps) -> Element {
    rsx! {
        toast::ToastCloseButton {
            class: "toast-close",
            attributes: props.attributes,
            children: props.children,
        }
    }
}

#[component]
pub fn ToastProvider(
    #[props(default = ReadSignal::new(Signal::new(Some(Duration::from_secs(5)))))]
    default_duration: ReadSignal<Option<Duration>>,
    #[props(default = ReadSignal::new(Signal::new(10)))] max_toasts: ReadSignal<usize>,
    #[props(default)] render_toast: Option<Callback<ToastPropsWithOwner, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let render_toast = render_toast
        .unwrap_or_else(|| Callback::new(|p: ToastPropsWithOwner| rsx! { StyledToast { ..p } }));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        toast::ToastProvider {
            class: "toast-container",
            default_duration,
            max_toasts,
            render_toast,
            attributes,
            {children}
        }
    }
}
