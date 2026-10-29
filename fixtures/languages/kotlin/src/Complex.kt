interface Shape { fun area(): Double }
class Circle(val r: Double) : Shape { override fun area() = Math.PI * r * r }
fun total(shapes: List<Shape>) = shapes.sumOf { it.area() }
