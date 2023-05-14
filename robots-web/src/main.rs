cfg_if::cfg_if! {
if #[cfg(feature = "ssr")] {
    mod uart;
    mod queues;
    mod http;
    mod error;

    #[actix_web::main]
    async fn main() {
        println!("main start");
        //http::serve().await;
        let http_serve = http::serve();
        let uart_serve = uart::serve();
        tokio::pin!(http_serve);
        tokio::pin!(uart_serve);
        println!("main serve");
        tokio::select! {
            ret = http_serve => eprintln!("http::serve ended with {ret:?}"),
            ret = uart_serve => eprintln!("uart::serve ended with {ret:?}"),
        }
    }
} else {
    pub fn main() {
        // no client-side main function
        // unless we want this to work with e.g., Trunk for pure client-side testing
        // see lib.rs for hydration function instead
    }
}
}
