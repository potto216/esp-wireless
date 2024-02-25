# Espressif wireless examples

This repository contains Espressif wireless examples in Rust.

## Background

For the example I used code from the [std-training book](https://esp-rs.github.io/std-training) specifically
[http_server.rs](https://github.com/esp-rs/std-training/blob/main/intro/http-server/examples/http_server.rs)
and from [esp-idf-svc http_server.rs](https://github.com/esp-rs/esp-idf-svc/blob/master/examples/http_server.rs)

## Setup and Running

The following was run in a Ubuntu Linux VM, specifically Linux esp-rs-dev 6.5.0-21-generic #21~22.04.1-Ubuntu. I used this [ansible recipe](https://github.com/potto216/automation-management/blob/main/playbooks/setup_general_comm_dev.yaml) with the following command to setup the rust development environment.

```
ansible-playbook  -vv ./playbooks/setup_general_comm_dev.yaml --vault-password-file ./vault/vault_password.txt -l esp-rs-dev-01 --extra-vars "install_python=true install_vim=true install_rust=true"
```

Then ran the following commands manually to clone the repos and configure the nightly build.
```
# setup the paths and clone the repos. 
git clone https://github.com/potto216/esp-wireless
cd esp-wireless

# common files are needed
git clone https://github.com/esp-rs/std-training.git

# just cloned this for quick reference of the API
git clone https://github.com/esp-rs/esp-idf-svc.git

# Configure your Wi-Fi settings in cfg.toml. Don't commit it.
cp cfg.toml.example cfg.toml
vi cfg.toml

# Make sure you are a member of the dialout group to use the USB serial port
sudo usermod -a -G dialout user

# I was not able to install libuv1-dev or libuv-dev initially
sudo apt install llvm-dev libclang-dev clang  pkgconf python3-venv python-is-python3

sudo vi /etc/apt/sources.list

# Edit the /etc/apt/sources.list to use the deb-src. Example:
#=====START /etc/apt/sources.list===============
deb http://archive.ubuntu.com/ubuntu jammy main universe restricted multiverse
deb-src http://archive.ubuntu.com/ubuntu jammy main universe restricted multiverse #Added by software-properties
deb http://security.ubuntu.com/ubuntu/ jammy-security universe restricted multiverse main
deb-src http://security.ubuntu.com/ubuntu/ jammy-security universe restricted multiverse main #Added by software-properties
deb http://archive.ubuntu.com/ubuntu jammy-updates universe restricted multiverse main
deb-src http://archive.ubuntu.com/ubuntu jammy-updates universe restricted multiverse main #Added by software-properties
deb http://archive.ubuntu.com/ubuntu jammy-backports universe restricted multiverse main
deb-src http://archive.ubuntu.com/ubuntu jammy-backports universe restricted multiverse main #Added by software-properties
deb http://archive.ubuntu.com/ubuntu jammy-proposed universe restricted multiverse main
#=====END /etc/apt/sources.list===============

sudo apt update
sudo apt upgrade
sudo apt install libuv1-dev
rustup toolchain install nightly-2023-11-14 --component rust-src

sudo apt install librust-libudev-dev
cargo install cargo-espflash espflash ldproxy

# plug in the ESP32-C3 Board and verify the serial port shows up
ls /dev/tty*

cd http-server-full
cargo build
cargo run
```

Now you should be able to use the browser on another computer and access the pages with http, not https
