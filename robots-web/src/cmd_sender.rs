use leptos::*;

use robots_lib::Cmd;

#[cfg(feature = "ssr")]
use crate::{error::Error, queues::TX};

#[server(SendCmd, "/api", "Cbor")]
pub async fn send_cmd(cmd: Cmd) -> Result<(), ServerFnError> {
    println!("hello {cmd:?}...");
    Ok(TX.0.send(cmd).await.map_err(Error::SendError)?)
}

#[component]
pub fn CmdSender(cx: Scope) -> impl IntoView {
    let cmd_sender = create_action(cx, |cmd: &Cmd| send_cmd(*cmd));

    view! { cx,
        <button on:click=move |_| {cmd_sender.dispatch(Cmd::Get)}>"Get"</button>
        <button on:click=move |_| {cmd_sender.dispatch(Cmd::Ping)}>"Ping"</button>
        <button on:click=move |_| {cmd_sender.dispatch(Cmd::Pong)}>"Pong"</button>
    }
}
