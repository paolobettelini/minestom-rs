# Minestom Rust bindings.

This crate is not meant to be used in a binary crate. [Minestom](https://github.com/Minestom/Minestom) needs to run in a JVM.
You need to create both a (minimal) Java project and a `cdylib` crate.
The Java project will load the `cdylib` and execute your code.
This repository also contains bindings for [Minestom](https://github.com/AtlasEngineCa/WorldSeedEntityEngine).

## Java-side

Create a Java applicaton project and import this library.
Supose `org.example.Main` is your `Main` class.
```java
package org.example;

public class Main {

    static {
        try {
            String libraryPath = System.getProperty("java.library.path");
            System.loadLibrary("minestom");
        } catch (UnsatisfiedLinkError e) {
            System.err.println("Failed to load native library: " + e.getMessage());
            e.printStackTrace();
        }
    }

    public static native void startServer();

    public static void main(String[] args) {
        startServer();
    }

}
```

The `startServer()` method will you be your entry point
for the rust code.

## Rust-side

Create a `lib` crate
```toml
[dependencies]
minestom = { git = "https://github.com/paolobettelini/minestom-rs" }
world-seed-entity-engine = { git = "https://github.com/paolobettelini/minestom-rs" } # optional

[lib]
name = "minestom"
crate-type = ["cdylib"] 
```
The entry point will be in `lib.rs`.
```rust
#[unsafe(no_mangle)]
pub extern "system" fn Java_rust_minestom_Main_startServer(env: JNIEnv, class: JClass) -> jint {
    // Attach JVM to the current thread
    minestom::init_runtime();
    let res = minestom::jni_utils::attach_jvm(&env);

    match res {
        Ok(_) => (),
        Err(e) => {
            eprintln!("failed to attach JVM: {}", e);
            return -1;
        }
    }

    let result = panic::catch_unwind(|| {
        minestom::RUNTIME.block_on(async {
            match run_server().await {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("server error: {}", e);
                    -1
                }
            }
        })
    });

    match result {
        Ok(code) => code,
        Err(_) => {
            eprintln!("panic in Rust server");
            -1
        }
    }
}
```
where `run_server` is your main function.

## Running
Compile the crate, which will generate the library in the `target` folder.
Assume `$CARGO_TARGET_DIR/release` is the `target` folder.
Run the jar with `-Djava.library.path=$CARGO_TARGET_DIR/release`.
Add `--add-opens java.base/java.lang=ALL-UNNAMED` if you are using `WorldSeedEntityEngine`.
```bash
gradle build
java -Djava.library.path=$CARGO_TARGET_DIR/release -jar build/libs/app.jar 
```

# TODO

Event and command callbacks should be async. InstanceContainer, SharedInstance etc dovrebbero avere gli stessi metodi comuni.
Lo scheduler non funziona, dà errore JNI.