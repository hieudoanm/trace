import shapes.*

fun main() {
    val shapes: List<Shape> = listOf(
        Circle(2.0),
        Rectangle(3.0, 4.0),
        Square(5.0)
    )
    println("Total area: ${"%.2f".format(totalArea(shapes))}")
}
