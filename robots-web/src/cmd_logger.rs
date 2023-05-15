use chrono::prelude::*;
use leptos::*;

use robots_lib::Cmd;

#[cfg(feature = "ssr")]
use robots_drv::TX;

#[server(SendCmd, "/api", "Cbor")]
pub async fn send_cmd(cmd: Cmd) -> Result<(), ServerFnError> {
    TX.send(&cmd)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[component]
pub fn CmdLogger(cx: Scope) -> impl IntoView {
    let (cmds, set_cmds) = create_signal::<Vec<(DateTime<Utc>, Cmd)>>(cx, vec![]);
    let (scmds, set_scmds) = create_signal::<Vec<(DateTime<Utc>, Cmd)>>(cx, vec![]);

    let cmd_sender = create_action(cx, move |cmd: &Cmd| {
        set_scmds.update(|cmds| cmds.push((Utc::now(), *cmd)));
        send_cmd(*cmd)
    });

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
                        set_cmds.update(|cmds| cmds.push((Utc::now(), v)));
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

    let button = "text-slate-100 p-2 m-4 rounded-br-lg \
                  bg-sky-700 hover:bg-sky-600 active:bg-sky-500 \
                  shadow-lg hover:shadow-xl active:shadow-2xl";

    view! { cx,
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Get)}>"Get"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Ping)}>"Ping"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Pong)}>"Pong"</button>
        <br />
        <div class="flex text-slate-100">
            <ol class="flex-auto">
                <li class="underline">"Sent"</li>
                <For each=cmds key=|cmd| cmd.0 view=move |cx, (dt, cmd)| {
                    view! {
                        cx,
                        <li>{format!("{dt:?} {cmd:?}")}</li>
                    }
                }/>
            </ol>
            <ol class="flex-auto">
                <li class="underline">"Received"</li>
                <For each=scmds key=|cmd| cmd.0 view=move |cx, (dt, cmd)| {
                    view! {
                        cx,
                        <li>{format!("{dt:?} {cmd:?}")}</li>
                    }
                }/>
            </ol>
        </div>
    }
}
