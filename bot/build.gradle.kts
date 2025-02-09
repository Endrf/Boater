import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    application
    kotlin("jvm") version "2.1.0"
    id("org.jetbrains.kotlin.plugin.serialization") version "1.9.22"
    id("com.github.johnrengelman.shadow") version "7.0.0"
    id("app.cash.sqldelight") version "2.0.2"
}

application.mainClass = "com.endrf.boater.BotKt"
group = "com.endrf"
version = "0.4.0"

val jdaVersion = "5.1.0"

repositories {
    mavenCentral()
    maven { url = uri("https://jitpack.io") }
}

dependencies {
    testImplementation(kotlin("test"))
    implementation("net.dv8tion:JDA:$jdaVersion")
    implementation("dev.arbjerg:lavaplayer:2.2.1")
    implementation("com.squareup.okhttp3:okhttp:5.0.0-alpha.14")
    implementation("io.ktor:ktor-server-netty:2.3.12")
    implementation("io.ktor:ktor-server-websockets:2.3.11")
    implementation("io.ktor:ktor-serialization-kotlinx-json:2.3.11")
    implementation("org.reflections:reflections:0.10.2")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.3")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1-Beta")
    implementation("com.github.dotenv-org:dotenv-vault-kotlin:0.0.3")
    implementation("net.peanuuutz.tomlkt:tomlkt:0.4.0")
    implementation("app.cash.sqldelight:sqlite-driver:2.0.2")
    implementation("com.squareup.moshi:moshi-kotlin:1.15.1")
}

sqldelight {
    databases {
        create("BoaterData") {
            packageName.set("com.endrf")
        }
    }
}

tasks.test {
    useJUnitPlatform()
}

tasks.withType<KotlinCompile> {
    kotlinOptions.jvmTarget = "18"
}