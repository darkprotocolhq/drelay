use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web::web::block;
use log::{error, info};
use solana_client::rpc_client::RpcClient;
use std::sync::Mutex;

struct AppState {
    rpc_status: Mutex<String>,
}

async fn index(data: web::Data<AppState>) -> impl Responder {
    let rpc_status = data.rpc_status.lock().unwrap();
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("Dark Protocol Relayer - Operational [ONLINE] - RPC Status: {}", *rpc_status))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,info");
    env_logger::init();

    let rpc_url = "https://devnet.helius-rpc.com/?api-key=b512dcfe-ee18-42ea-b3c7-82dab825d317";
    let rpc_client = RpcClient::new(rpc_url.to_string());

    let initial_rpc_status = check_rpc_connection(rpc_client).await; // Pass owned RpcClient

    let app_state = web::Data::new(AppState {
        rpc_status: Mutex::new(initial_rpc_status),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(web::resource("/").to(index))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

async fn check_rpc_connection(rpc_client: RpcClient) -> String {  // Take ownership
    match block(move || rpc_client.get_version()).await {
        Ok(version_info) => {
            let version = version_info.unwrap().solana_core;
            let msg = format!("Connected to Solana RPC. Version: {}", version);
            info!("{}", msg);
            msg
        },
        Err(e) => {
            let msg = format!("Failed to connect to Solana RPC: {:?}", e);
            error!("{}", msg);
            msg
        },
    }
}
