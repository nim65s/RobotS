use crate::error::Result;
use robots_web::cmd_sender::SendCmd;

use actix_files::Files;
use actix_web::*;
use leptos::*;
use leptos_actix::{generate_route_list, LeptosRoutes};

use robots_web::app::*;

pub fn register_server_functions() {
    _ = SendCmd::register();
}

pub async fn serve() -> Result<()> {
    println!("http start");
    register_server_functions();

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> });

    let server = HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                |cx| view! { cx, <App/> },
            )
            .service(Files::new("/", site_root))
            .wrap(middleware::Compress::default())
    })
    .bind(&addr)?;
    println!("http serve");
    server.run().await?;
    Ok(())
}
