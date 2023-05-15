use leptos::*;

use robots_lib::Cmd;

#[component]
pub fn CmdLogger(cx: Scope) -> impl IntoView {
    let mut id = 0;
    let (cmds, set_cmds) = create_signal::<Vec<(usize, Cmd)>>(cx, vec![(id, Cmd::Ping)]);

    cfg_if::cfg_if! {
    if #[cfg(not(feature = "ssr"))] {
        use futures::StreamExt;
        use gloo_net::eventsource::futures::EventSource;

        let mut source = EventSource::new("/api/sse").expect("couldn't connect to SSE stream");
        create_signal_from_stream(
            cx,
            source.subscribe("msg").unwrap().map(move |v| match v {
                Err(e) => format!("sse connection error: {e:?}"),
                Ok((_, v)) => match Cmd::from_sse(&v) {
                    Err(e) => format!("sse decoding error: {e:?}"),
                    Ok(Some(v)) => {
                        set_cmds.update(|cmds| cmds.push((id, v)));
                        id += 1;
                        log!("got {v:?}");
                        format!("{v:?}")
                    }
                    v => format!("{v:?}"),
                },
            }),
        );

        on_cleanup(cx, move || source.close());
    }
    };

    view! { cx,
        <ol>
        <For each=cmds key=|cmd| cmd.0 view=move |cx, (_, cmd)| {
            view! {
                cx,
                <li>"Received: " {format!("{cmd:?}")}</li>
            }
        }/>
        </ol>
    }
}
