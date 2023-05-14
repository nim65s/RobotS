use leptos::*;

use robots_lib::Cmd;

#[cfg(feature = "ssr")]
use crate::{error::Error, queues::TX};

#[component]
pub fn CmdLogger(cx: Scope) -> impl IntoView {
    let mut next_id = 0;
    let (cmds, set_cmds) = create_signal(cx, Vec::<(u8, Cmd)>::new());
    let on_click = move |_| {
        set_cmds.update(|cmds| {
            cmds.push((next_id, Cmd::Ping));
            next_id += 1;
        })
    };

    view! { cx,
    <button on:click=on_click>"Ping"</button>
        <ul>
        <For
        each=cmds
        key=|cmd| cmd.0
        view=move |cx, cmd| {
            view! { cx, <li>{cmd.0}</li> }
        }
    />
    </ul>
    }
}
