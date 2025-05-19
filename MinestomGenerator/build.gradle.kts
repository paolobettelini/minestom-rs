plugins {
    id("java")
    id("com.github.johnrengelman.shadow") version "8.1.1"
    application
}

group = "org.example"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
    maven("https://jitpack.io")
}

dependencies {
    // Direct implementation from GitHub main branch
    implementation("com.github.Minestom:Minestom:master-SNAPSHOT")
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
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
} 