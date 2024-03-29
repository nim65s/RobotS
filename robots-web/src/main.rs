cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use actix_files::Files;
        #[allow(clippy::wildcard_imports)]
        use actix_web::*;
        #[allow(clippy::wildcard_imports)]
        use leptos::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use futures::StreamExt;

        use robots_drv::{RX, driver, get_port};

        //use robots_web::cmd_logger::SendCmd;
        use robots_web::app::App;
        use robots_web::error::{Error, Result};

        #[get("/api/sse")]
        async fn uart_rx_to_sse() -> impl Responder {
            HttpResponse::Ok()
                .insert_header(("Content-Type", "text/event-stream"))
                .streaming(RX.clone().map(|value| value.as_sse("msg")))
        }

        #[actix_web::main]
        async fn main() -> Result<()> {
            println!("main start");

            // setup uart
            let uart_port = serialport::new(get_port()?, 115_200);
            if let Err(e) = driver(uart_port) {
                eprintln!("uart driver error: {e:?}");
                return Err(Error::UartDriver)
            }

            // setup http
            let conf = get_configuration(None).await?;
            let addr = conf.leptos_options.site_addr;

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|| view! {  <App/> });

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;

                App::new()
                    .service(uart_rx_to_sse)
                    .leptos_routes(
                        leptos_options.to_owned(),
                        routes.clone(),
                        || view! {  <App/> },
                        )
                    .service(Files::new("/", site_root))
                    //.wrap(middleware::Compress::default())
            })
            .bind(&addr)?.run().await?;
            Ok(())
        }
    } else {
        pub fn main() {
            // no client-side main function
            // unless we want this to work with e.g., Trunk for pure client-side testing
            // see lib.rs for hydration function instead
        }
    }
}
