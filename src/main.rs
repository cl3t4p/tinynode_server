use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tracing::{error, info};




#[derive(Clone)]
struct AppState{
    mqtt: AsyncClient,
    // Simple shared secret for API auth , can be replaced after
    api_token: String,
}

#[derive(Deserialize)]
struct RelayReq {
    // Accept either 0/1 or true/false. Keep it simple for now: 0/1.
    state: u8,
    port: u8,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // -- Config via env vars --
    let server_address = std::env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:7538".to_string());

    let api_token = std::env::var("API_TOKEN").unwrap_or_else(|_| "devtoken".to_string());

    let mqtt_host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let mqtt_port = std::env::var("MQTT_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(1883);

    let mqtt_username = std::env::var("MQTT_USER").ok();
    let mqtt_password = std::env::var("MQTT_PASS").ok();

    // -- MQTT Client setup --
    let mut mqttoptions = MqttOptions::new("rust-api",mqtt_host,mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(30));

    if let (Some(username), Some(password)) = (mqtt_username, mqtt_password) {
        mqttoptions.set_credentials(username, password);
    }

    let (mqtt, eventloop) = AsyncClient::new(mqttoptions,10);
    spawn_mqtt_eventloop(eventloop);

    let state = AppState{mqtt, api_token: api_token.to_string()};


    let app = Router::new()
        //.route("/", get(|| async { "Hello, world!" }))
        .route("/device/{id}/relay",post(set_relay))
        .with_state(Arc::new(state));

    let addr: SocketAddr = server_address.parse()?;

    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener,app).await?;

    Ok(())
}


fn spawn_mqtt_eventloop(mut eventloop: EventLoop){
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(_notification) => {
                    //You can log or ignore. every packet
                }
                Err(error) => {
                    error!("MQTT connection error: {error:?}");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    });
}

async fn set_relay(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<RelayReq>,
)-> impl IntoResponse {
    let Some(auth) = headers.get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok()) else{
        return (StatusCode::UNAUTHORIZED,"missing authorization").into_response();
    };
    let expected = format!("Bearer {}",state.api_token);
    if auth != expected {
        return (StatusCode::UNAUTHORIZED,"incorrect authentication").into_response();
    }

    let s = match req.state {
        0 => 0u8, // Off
        1 => 1u8, // On
        2 => 2u8, // On with predefined timer
        _ => return (StatusCode::BAD_REQUEST, "state must be [0,1,2]").into_response(),
    };
    //TODO Implement port check



    let topic = format!("devices/{}/relay/set",id);
    let payload: Vec<u8> = vec![s, req.port]; // bytes 0.
    match state
        .mqtt
        .publish(topic,QoS::ExactlyOnce,false,payload)
        .await
    {
        Ok(_) => (StatusCode::OK,"OK").into_response(),
        Err(error) => {
            error!("MQTT publish error: {error:?}");
            (StatusCode::BAD_GATEWAY, "mqtt publish failed").into_response()
        }
    }
}