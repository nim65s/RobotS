cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use actix_files::Files;
        use actix_web::*;
        use leptos::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use futures::StreamExt;

        use robots_drv::{RX, driver};

        //use robots_web::cmd_logger::SendCmd;
        use robots_web::app::*;

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
            let port = option_env!("ROBOTS_PORT").unwrap_or("/dev/ttyUSB0");
            let uart_port = serialport::new(port, 115_200);
            driver(uart_port).expect("uart driver error");

            // setup http
            let conf = get_configuration(None).await.unwrap();
            let addr = conf.leptos_options.site_addr;

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|| view! {  <App/> });

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;

                App::new()
                    .service(uart_rx_to_sse)
                    .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
                    .leptos_routes(
                        leptos_options.to_owned(),
                        routes.to_owned(),
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
