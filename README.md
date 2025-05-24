# WIP Minestom rust bindings.

## How to run
Compile `example-server` which will generate the `.so`.
Then
```bash
./gradlew build && java -Djava.library.path=/home/paolo/Desktop/rust_target/debug -jar build/libs/app.jar 
```

## Texture pack
```bash
cd resourcepack
zip -r ../resourcepack.zip *
```

## TODO
Event and command callbacks should be async. InstanceContainer, SharedInstance etc dovrebbero avere gli stessi metodi comuni.
Lo scheduler non funziona,dà errore JNI.