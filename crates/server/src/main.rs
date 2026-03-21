use salvo::catcher::Catcher;
use salvo::conn::rustls::{Keycert, RustlsConfig};
use salvo::prelude::*;
use salvo::server::ServerHandle;
use tokio::signal;
use tracing::log::info;


#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");
    infrastructure::config::init();
    let config = infrastructure::config::get();
    // db::init(&config.db).await; //TODO Uncomment
    // smtp init
    // s3 init
    let _guard = infrastructure::log::init(&config.log); // todo change to init
    tracing::info!("log level: {}", &config.log.filter_level);


    let service = Service::new(routers::root())  //TODO Uncomment
        .catcher(Catcher::default().hoop(error404::error_404))
        .hoop(cors_hoop());
    println!("🔄 listen on {}", &config.listen_addr);
    println!("Debug: TLS config is {:?}", config.tls); // Add this for debug only
    if let Some(tls) = &config.tls {
        let listen_addr = &config.listen_addr;
        println!(
            "📖 Open API Page (test Quinn): https://{}/scalar",
            listen_addr.replace("0.0.0.0", "127.0.0.1")
        );

        let config = RustlsConfig::new(
            Keycert::new()
                .cert_from_path(&tls.cert).expect("failed to read cert file")
                .key_from_path(&tls.key).expect("failed to read key file"),
        );
        let acceptor = QuinnListener::new(config.clone().build_quinn_config().unwrap(),listen_addr).join(TcpListener::new(listen_addr).rustls(config)).bind().await;
        let server = Server::new(acceptor);
        tokio::spawn(shutdown_signal(server.handle()));
        server.serve(service).await; //TODO Uncomment
    } else {
        println!(
            "📖 Open API Page: http://{}/scalar",
            config.listen_addr.replace("0.0.0.0", "127.0.0.1")
        );

        let acceptor =TcpListener::new(&config.listen_addr).bind().await;
        let server = Server::new(acceptor);
        tokio::spawn(shutdown_signal(server.handle()));
        server.serve(service).await;  //TODO Uncomment
    }



}

async fn shutdown_signal(handle: ServerHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("ctrl_c signal received"),
        _ = terminate => info!("terminate signal received"),
    }
    handle.stop_graceful(std::time::Duration::from_secs(60));
}





use salvo::cors::{AllowHeaders, AllowMethods, AllowOrigin, Cors, CorsHandler};


pub fn cors_hoop() -> CorsHandler {
    Cors::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
        .into_handler()
}