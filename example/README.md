# Example server

- `Relay` the central server
- `Server Container` A minecraft server
- `Game Server` A virtual server handled by a server container

## Running
Start the nats server
```bash
nats-server
```
Start the http server
```bash
cd resources/output
miniserve . --port 6543 --verbose
```
Compile both libraries
```bash
cd crates/thecrown-auth
cargo b -r
cd ../thecrown-mcserver
cargo b -r
```
Compile the jar
```bash
gradle build
```
Start the servers
```bash
java -Dlib.name=minestom -Djava.library.path=$CARGO_TARGET_DIR/debug --add-opens java.base/java.lang=ALL-UNNA -jar app/build/libs/app.jar
java -Dlib.name=auth -Djava.library.path=$CARGO_TARGET_DIR/debug --add-opens java.base/java.lang=ALL-UNNA -jar app/build/libs/app.jar
```