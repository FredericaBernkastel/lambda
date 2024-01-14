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
```
### Run with docker
```bash
# build and run the container
docker compose up --build

# open the running container console, and register new admin user
cd app && ./lambda register -u "user" -p "password"
```
Your application will be available at http://localhost:8080/views/home.  
For more docker options, see [`readme.Docker.md`](README.Docker.md).

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
