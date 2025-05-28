plugins {
    application
    id("com.github.johnrengelman.shadow") version "8.1.1"
}

repositories {
    mavenCentral()
}

dependencies {
    implementation(files("../../../MinestomRust/build/libs/minestom-library.jar"))
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(21)
    }
}

tasks.named("startScripts") {
    dependsOn(tasks.shadowJar)
}

tasks.named("build") {
    dependsOn(tasks.shadowJar)
}

tasks.shadowJar {
    archiveBaseName.set("app")
    archiveClassifier.set("")
    archiveVersion.set("")
    manifest {
        attributes["Main-Class"] = "net.thecrown.App"
    }
    mergeServiceFiles()
    from(zipTree("../../../MinestomRust/build/libs/minestom-library.jar"))
}

tasks.jar {
    enabled = false
}

application {
    mainClass.set("net.thecrown.App")
}
