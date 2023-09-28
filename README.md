## Installation of building/flashing requirements
### ESP32
[ESP Rust Book section](https://esp-rs.github.io/book/installation/riscv-and-xtensa.html).


```
cargo install ldproxy
cargo install espflash
cargo install espup  # requires make for openssl

espup install

# consider copy paste export statements
# to your .bashrc, .zshrc or equivalent.
source ~/export-esp.sh
```

## Pluggin in behind reverse proxy
Here is an example Nginx configuration:
```nginx
server {
  listen 443 ssl http2;
  listen [::]:443 ssl http2;
  server_name mydomain.com;

  ssl_certificate /etc/letsencrypt/live/mydomain.com/fullchain.pem;
  ssl_certificate_key /etc/letsencrypt/live/mydomain.com/privkey.pem;

  client_max_body_size 1M;

  location / {
    # npm run preview --host 127.0.0.1
    proxy_pass http://127.0.0.1:4173/;
  }
  location /api/ {
    # cargo run
    proxy_set_header X-Real-IP $remote_addr;
    proxy_pass http://127.0.0.1:8080/;
  }
}
```
