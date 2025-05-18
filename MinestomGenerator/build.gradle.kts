plugins {
    id("java")
    id("com.github.johnrengelman.shadow") version "8.1.1"
    application
}

group = "org.example"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
    maven(url = "https://jitpack.io")
}

dependencies {
    implementation("net.minestom:minestom-snapshots:7320437640")
}

application {
    mainClass.set("org.example.Main")
}

tasks.shadowJar {
    archiveBaseName.set("app")
    archiveClassifier.set("")
    archiveVersion.set("")
    manifest {
        attributes(
            "Main-Class" to "org.example.Main"
        )
    }
    mergeServiceFiles()
}

tasks.build {
    dependsOn(tasks.shadowJar)
}

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
} 