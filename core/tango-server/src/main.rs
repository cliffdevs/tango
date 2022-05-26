mod httputil;
mod iceconfig;
mod signaling;
use envconfig::Envconfig;
use prost::Message;
use routerify::ext::RequestExt;

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "LISTEN_ADDR", default = "[::]:1984")]
    listen_addr: String,

    // Don't use this unless you know what you're doing!
    #[envconfig(from = "USE_X_REAL_IP", default = "false")]
    use_x_real_ip: bool,
}

struct State {
    real_ip_getter: httputil::RealIPGetter,
    iceconfig_server: std::sync::Arc<iceconfig::Server>,
    signaling_server: std::sync::Arc<signaling::Server>,
}

async fn handle_iceconfig_request(
    request: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, anyhow::Error> {
    let state = request.data::<State>().unwrap();
    let remote_ip = state
        .real_ip_getter
        .get_remote_real_ip(&request)
        .ok_or(anyhow::anyhow!("could not get remote ip"))?;
    let iceconfig_server = state.iceconfig_server.clone();
    let req = tango_protos::iceconfig::GetRequest::decode(
        hyper::body::to_bytes(request.into_body()).await?,
    )?;
    log::debug!("/iceconfig: {:?}", req);
    Ok(hyper::Response::builder()
        .header(hyper::header::CONTENT_TYPE, "application/x-protobuf")
        .body(
            iceconfig_server
                .get(&remote_ip)
                .await?
                .encode_to_vec()
                .into(),
        )?)
}

async fn handle_iceconfig_legacy_request(
    request: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, anyhow::Error> {
    let state = request.data::<State>().unwrap();
    let remote_ip = state
        .real_ip_getter
        .get_remote_real_ip(&request)
        .ok_or(anyhow::anyhow!("could not get remote ip"))?;
    let iceconfig_server = state.iceconfig_server.clone();
    let req = tango_protos::iceconfig::GetRequest::decode(
        hyper::body::to_bytes(request.into_body()).await?,
    )?;
    log::debug!("/relay: {:?}", req);
    Ok(hyper::Response::builder()
        .header(hyper::header::CONTENT_TYPE, "application/x-protobuf")
        .body(
            iceconfig_server
                .get_legacy(&remote_ip)
                .await?
                .encode_to_vec()
                .into(),
        )?)
}

async fn handle_signaling_request(
    mut request: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, anyhow::Error> {
    if !hyper_tungstenite::is_upgrade_request(&request) {
        return Ok(hyper::Response::builder()
            .status(hyper::StatusCode::BAD_REQUEST)
            .body(
                hyper::StatusCode::BAD_REQUEST
                    .canonical_reason()
                    .unwrap()
                    .into(),
            )?);
    }

    let (response, websocket) = hyper_tungstenite::upgrade(
        &mut request,
        Some(tungstenite::protocol::WebSocketConfig {
            max_message_size: Some(4 * 1024 * 1024),
            max_frame_size: Some(1 * 1024 * 1024),
            ..Default::default()
        }),
    )?;

    let signaling_server = request.data::<State>().unwrap().signaling_server.clone();
    tokio::spawn(async move {
        let websocket = match websocket.await {
            Ok(websocket) => websocket,
            Err(e) => {
                log::error!("error in websocket connection: {}", e);
                return;
            }
        };
        if let Err(e) = signaling_server.handle_stream(websocket).await {
            log::error!("error in websocket connection: {}", e);
        }
    });

    Ok(response)
}

fn router(
    real_ip_getter: httputil::RealIPGetter,
    iceconfig_backend: Option<Box<dyn iceconfig::Backend + Send + Sync + 'static>>,
) -> routerify::Router<hyper::Body, anyhow::Error> {
    routerify::Router::builder()
        .data(State {
            real_ip_getter,
            iceconfig_server: std::sync::Arc::new(iceconfig::Server::new(iceconfig_backend)),
            signaling_server: std::sync::Arc::new(signaling::Server::new()),
        })
        .get("/", handle_signaling_request)
        .get("/signaling", handle_signaling_request)
        .post("/iceconfig", handle_iceconfig_request)
        .post("/relay", handle_iceconfig_legacy_request)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_default_env()
        .filter(Some("tango_server"), log::LevelFilter::Info)
        .init();
    log::info!("welcome to tango-server {}!", git_version::git_version!());
    let config = Config::init_from_env().unwrap();
    let real_ip_getter = httputil::RealIPGetter::new(config.use_x_real_ip);
    let iceconfig_backend: Option<Box<dyn iceconfig::Backend + Send + Sync + 'static>> = {
        log::warn!("no iceconfig backend, will not service iceconfig requests");
        None
    };
    let addr = config.listen_addr.parse()?;
    let router = router(real_ip_getter, iceconfig_backend);
    let service = routerify::RouterService::new(router).unwrap();
    hyper::Server::bind(&addr).serve(service).await?;
    Ok(())
}
