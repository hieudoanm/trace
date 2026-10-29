import io.ktor.server.netty.embeddedServer
import io.ktor.server.engine.*

fun main() {
    embeddedServer(Netty, port = 8080) {
        routing {
            get("/hello") {
                call.respondText("Hello Ktor")
            }
        }
    }.start(wait = true)
}
