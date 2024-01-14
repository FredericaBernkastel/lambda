![Docker Image CI](https://github.com/FredericaBernkastel/lambda/actions/workflows/docker-image.yml/badge.svg)  
An experiment of building a web backend with Rust and Docker.

### Run yourself
```bash
# compile (requires nightly toolchain)
cargo build --release
mv target/release/lambda deploy/lambda

# register new admin user (cli)
cd deploy && ./lambda register -u "user" -p "password"

# run server
./lambda
# login at http://127.0.0.1:8080/views/login
```
For running with docker, see [`readme.Docker.md`](readme.Docker.md).

### Database ERD
![](doc/database_erd.png)

### Screenshots
`- Homepage`
![](doc/scr/homepage.jpg)

`- Graffiti page`
![](doc/scr/graffiti%20page.jpg)

`- Graffiti edit`
![](doc/scr/graffiti%20edit.png)

`- Author page`
![](doc/scr/author%20page.jpg)

`- Advanced graffiti search`
![](doc/scr/advanced%20graffiti%20search.png)

`- Backend error reporting`
![](doc/scr/backend%20error%20reporting.png)
