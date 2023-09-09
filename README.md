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
