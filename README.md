# TermdrawServer

A server which enables multiple `termdrawclient`s (not to be confused with `termdraw`)
to draw together on a single canvas.

## Hosting

Just clone this repository and use the provided `docker-compose.yml` or compile and
run.

```sh
# Docker
git clone https://github.com/enokiun/termdrawserver
cd termdrawserver
docekr-compose up
```

```sh
# Cargo
git clone https://github.com/enokiun/termdrawserver
cd termdrawserver
SERVER_ADDRESS="0.0.0.0" SERVER_PORT="8182" cargo run --release
```
