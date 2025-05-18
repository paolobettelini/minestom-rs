# WIP Minestom rust bindings.

## How to run
Compile `example-server` which will generate the `.so`.
Then
```bash
./gradlew build && java -Djava.library.path=/home/paolo/Desktop/rust_target/debug -jar build/libs/app.jar 
```