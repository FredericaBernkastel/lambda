An experiment of building a web backend with Rust.

### How to run
```bash
# compile (requires nightly toolchain)
cargo build --release
mv target/release/lambda deploy/lambda
# register new admin user (cli)
cd deploy && ./lambda register -u "user" -p "password"
# run
cd deploy && ./lambda
# login at
# http://127.0.0.1:8080/views/login
```