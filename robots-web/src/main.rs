cfg_if::cfg_if! {
if #[cfg(feature = "ssr")] {
    mod uart;
    mod queues;
    mod http;
    mod error;

    #[actix_web::main]
    async fn main() {
        tokio::select! {
            ret = http::serve() => eprintln!("http::serve ended with {ret:?}"),
            ret = uart::serve() => eprintln!("uart::serve ended with {ret:?}"),
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
