import org.gradle.api.GradleException
import org.gradle.api.tasks.JavaExec

plugins {
    application
    id("com.github.johnrengelman.shadow") version "8.1.1"
}

repositories {
    mavenCentral()
}

dependencies {
    implementation(files("../../../MinestomRust/build/libs/minestom-library.jar"))
    implementation(files("../../../WorldSeedEntityEngineRust/build/libs/wsee-library.jar"))
    implementation("commons-io:commons-io:2.11.0")
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
    //from(zipTree("../../../MinestomRust/build/libs/minestom-library.jar"))
    //from(zipTree("../../../MinestomRust/build/libs/wsee-library.jar"))
}

tasks.jar {
    enabled = false
}

application {
    mainClass.set("net.thecrown.App")
}

// ---------------------------------------------------------
// Custom JavaExec task: generatePack
// ---------------------------------------------------------
tasks.register<JavaExec>("generatePack") {
    group = "packbuilder"
    description = "Runs PackHelper.generate(bbmodelDir, resourcepackDir, modelsDir, mappings)"

    // the fully qualified main class
    mainClass.set("net.thecrown.PackHelper")

    // include your compiled classes + runtime deps
    classpath = sourceSets["main"].runtimeClasspath

    // pick up four paths from Gradle properties (-P)
    // you'll pass these in from the command line / build.rs
    val bbmodelPath: String = findProperty("bbmodelDir")
        ?.toString()
        ?: throw GradleException("Please supply -Pbbmodel=/path/to/bbmodelDir")
    val respackPath: String = findProperty("respackDir")
        ?.toString()
        ?: throw GradleException("Please supply -Prespack=/path/to/respackDir")
    val modelsPath: String = findProperty("modelsDir")
        ?.toString()
        ?: throw GradleException("Please supply -Pmodels=/path/to/modelsDir")
    val mappingsPath: String = findProperty("mappings")
        ?.toString()
        ?: throw GradleException("Please supply -Pmappings=/path/to/mappings")

    args = listOf(bbmodelPath, respackPath, modelsPath, mappingsPath)
}