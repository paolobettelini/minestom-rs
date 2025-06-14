plugins {
    `java-library`
    id("maven-publish")
    id("com.github.johnrengelman.shadow") version "8.1.1"
}

group = "rust.minestom"
version = "1.0.0"

repositories {
    mavenCentral()
    maven("https://jitpack.io")
}

dependencies {
    // Direct implementation from GitHub main branch
    //api("com.github.Minestom:Minestom:f62abc722f")
    implementation(files("/home/paolo/Downloads/Minestom/build/libs/minestom-dev.jar"))

    implementation("net.kyori:adventure-api:4.21.0")
    implementation("org.jetbrains:annotations:24.1.0")
}

java {
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
}

tasks.build {
    dependsOn(tasks.shadowJar)
}

tasks.jar {
    enabled = false
}

tasks.shadowJar {
    archiveBaseName.set("minestom-library")
    archiveClassifier.set("")
    archiveVersion.set("")
    mergeServiceFiles()
}