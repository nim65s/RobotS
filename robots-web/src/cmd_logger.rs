use leptos::*;

#[component]
pub fn CmdLogger(cx: Scope) -> impl IntoView {
    #[cfg(not(feature = "ssr"))]
    let latest = {
        use futures::StreamExt;
        use gloo_net::eventsource::futures::EventSource;

        let mut source = EventSource::new("/api/sse").expect("couldn't connect to SSE stream");
        let s = create_signal_from_stream(
            cx,
            source.subscribe("msg").unwrap().map(|v| match v {
                Err(e) => format!("sse connection error: {e:?}"),
                Ok((_, v)) => match Cmd::from_sse(&v) {
                    Err(e) => format!("sse decoding error: {e:?}"),
                    Ok(Some(v)) => format!("{v:?}"),
                    v => format!("{v:?}"),
                },
            }),
        );

        on_cleanup(cx, move || source.close());
        s
    };

    #[cfg(feature = "ssr")]
    let (latest, _) = create_signal(cx, "");

    view! { cx,
        <span>{latest}</span>
    }
}
