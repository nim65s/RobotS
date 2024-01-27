#![allow(clippy::unsafe_derive_deserialize)]
use chrono::prelude::*;
#[allow(clippy::wildcard_imports)]
use leptos::*;

use robots_lib::Cmd;

#[cfg(feature = "ssr")]
use robots_drv::TX;

#[cfg(not(feature = "ssr"))]
use futures::StreamExt;
#[cfg(not(feature = "ssr"))]
use gloo_net::eventsource::futures::EventSource;

#[server(SendCmd, "/api", "Cbor")]
pub async fn send_cmd(cmd: Cmd) -> Result<(), ServerFnError> {
    TX.send(&cmd)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn CmdLogger() -> impl IntoView {
    #[allow(unused_variables)]
    let (cmds, set_cmds) = create_signal::<Vec<(DateTime<Utc>, bool, Cmd)>>(vec![]);

    let cmd_sender = create_action(move |cmd: &Cmd| {
        set_cmds.update(|cmds| cmds.push((Utc::now(), true, *cmd)));
        send_cmd(*cmd)
    });

    cfg_if::cfg_if! {
    if #[cfg(not(feature = "ssr"))] {

        match EventSource::new("/api/sse") {
            Err(e) => eprintln!("couldn't connect to SSE stream: {e:?}"),
            Ok(mut source) => {
                match source.subscribe("msg") {
                    Err(e) => eprintln!("couldn't subscribe to 'msg' SSE stream: {e:?}"),
                    Ok(src) => {
                        create_signal_from_stream(
                            src.map(move |v| match v {
                                Err(e) => format!("sse connection error: {e:?}"),
                                Ok((_, v)) => match Cmd::from_sse(&v) {
                                    Err(e) => format!("sse decoding error: {e:?}"),
                                    Ok(Some(v)) => {
                                        set_cmds.update(|cmds| cmds.push((Utc::now(), false, v)));
                                        logging::log!("got {v:?}");
                                        format!("{v:?}")
                                    }
                                    v => format!("{v:?}"),
                                },
                            }),
                        );
                    }
                }

                on_cleanup( move || source.close());
            }
        }
    }};

    let button = "text-slate-100 p-2 m-4 rounded-br-lg \
                  bg-sky-700 hover:bg-sky-600 active:bg-sky-500 \
                  shadow-lg hover:shadow-xl active:shadow-2xl";

    view! {
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Hello)}>"Hello"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Ping)}>"Ping"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Pong)}>"Pong"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Button)}>"Button"</button>
        <br />
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Led(true))}>"LED ON"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Led(false))}>"LED OFF"</button>
        <br />
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Relay(true))}>"Relay ON"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Relay(false))}>"Relay OFF"</button>
        <br />
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Hue(0))}>"Red"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Hue(50))}>"Yellow"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Hue(100))}>"Green"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Hue(150))}>"Blue"</button>
        <button class={button} on:click=move |_| {cmd_sender.dispatch(Cmd::Hue(200))}>"Violet"</button>
        <br />
        <button class={button} on:click=move |_| {set_cmds.set(vec![])}>"Clear"</button>
        <br />
        <div class="grid grid-cols-3 text-slate-100">
            <div class="underline">"Received"</div>
            <div class="underline">"Timestamp"</div>
            <div class="underline">"Sent"</div>
            <For
                each=move || cmds.get()
                key=|cmd| cmd.0
                children=|(dt, sent, cmd)| view! {
                    <div><Show when=move || !sent>{format!("{cmd:?}")}</Show></div>
                    <div>{format!("{dt:?}")}</div>
                    <div><Show when=move || sent>{format!("{cmd:?}")}</Show></div>
                }
            />
        </div>
    }
}
