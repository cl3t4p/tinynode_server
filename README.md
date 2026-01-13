# TinyNode Rust Server (MQTT Relay Controller)

REST API server that publishes relay commands to an MQTT broker (Mosquitto).  
Intended flow: **Mobile/Web App → Axum HTTP → MQTT publish → ESP32 subscribes and executes**.

---

## Environment Variables

| Variable | Example | Notes |
|--------|--------|------|
| `SERVER_ADDRESS` | `0.0.0.0:9006` | Bind address for HTTP server |
| `API_TOKEN` | `******` | Used for Bearer auth |
| `MQTT_HOST` | `mosquitto` | Docker network hostname or IP |
| `MQTT_PORT` | `1883` | Mosquitto TCP listener |
| `MQTT_USER` | `rust_api` | Optional (required if `allow_anonymous false`) |
| `MQTT_PASS` | `******` | Optional |

Example `.env`:
```env
SERVER_ADDRESS=0.0.0.0:7538
API_TOKEN=secret_token
MQTT_HOST=mosquitto
MQTT_PORT=1883
MQTT_USER=rust_api
MQTT_PASS=supersecret
```

---

## Installation (Cargo)

### Prerequisites

- Rust toolchain (stable)
- Cargo
- An MQTT broker (e.g. Mosquitto) reachable from the server

### Build

```bash
cargo build --release
```

### Run

Option A: export variables in the shell:

```bash
export SERVER_ADDRESS=0.0.0.0:7538
export API_TOKEN=secret_token
export MQTT_HOST=127.0.0.1
export MQTT_PORT=1883
export MQTT_USER=rust_api
export MQTT_PASS=supersecret

./target/release/tinynode
```

Option B: use `.env` (recommended)

If you use a `.env` file, load it with your shell or a tool like `direnv`:

```bash
set -a
source .env
set +a

./target/release/tinynode
```

---


## Installation (Docker)

The server is published on Docker Hub:

https://hub.docker.com/repository/docker/cl3t4p/tinynode/general

### Pull image

```bash
docker pull cl3t4p/tinynode:latest
```

### Run container

```bash
docker run -d \
  --name tinynode \
  --env-file .env \
  -p 7538:7538 \
  cl3t4p/tinynode:latest
```

---

## Docker Compose Example

```yaml
services:
  mosquitto:
    image: eclipse-mosquitto:2
    container_name: mosquitto
    restart: always
    ports:
      - "1883:1883"   # MQTT (username/password)
      - "9001:9001"   # MQTT over WebSocket
    volumes:
      - /docker/mosquitto/config:/mosquitto/config
      - /docker/mosquitto/certs:/mosquitto/certs
      - /docker/mosquitto/log:/mosquitto/log
  tinynode:
    image: cl3t4p/tinynode:latest
    container_name: tinynode
    restart: always
    environment:
      - SERVER_ADDRESS=0.0.0.0:9006
      - API_TOKEN=*****
      - MQTT_HOST=10.0.0.1
      - MQTT_PORT=1883
      - MQTT_USER=rust_api
      - MQTT_PASS=*****
    ports:
      - "9006:9006"
    depends_on:
      - mosquitto

```

---

## API

### POST Relay Command

```
POST /device/{device_id}/relay
Authorization: Bearer <API_TOKEN>
Content-Type: application/json
```

Body:
```json
{
  "state": 1,
  "port": 16
}
```

---

## MQTT

- Topic: `devices/<device_id>/relay/set`
- Payload: 2 bytes `[state, port]`
- QoS: ExactlyOnce
- Retain: false

---
## Related Projects
- [TinyNode App](https://github.com/cl3t4p/tinynode_app).
- [ESP32 TinyNode Firmware](https://github.com/cl3t4p/tinynode_esp32).
