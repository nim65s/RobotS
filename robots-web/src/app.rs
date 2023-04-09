use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/robots-web.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <body class="bg-slate-800">
            <Router>
                <main>
                    <Routes>
                        <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                    </Routes>
                </main>
            </Router>
        </body>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    let button = "text-slate-100 p-2 my-4 rounded-br-lg \
                  bg-sky-700 hover:bg-sky-600 active:bg-sky-500 \
                  shadow-lg hover:shadow-xl active:shadow-2xl";

    view! { cx,
        <h1 class="text-white text-4xl my-4">"Welcome to Leptos!"</h1>
        <button class={button} on:click=on_click>"Click Me: " {count}</button>
    }
}
