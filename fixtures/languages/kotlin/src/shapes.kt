package shapes

// Shape is an interface for geometric shapes.
interface Shape {
    fun area(): Double
    fun perimeter(): Double
}

// Circle represents a circle with a given radius.
class Circle(val radius: Double) : Shape {
    override fun area(): Double {
        return Math.PI * radius * radius
    }

    override fun perimeter(): Double {
        return 2 * Math.PI * radius
    }
}

// Rectangle represents a rectangle with width and height.
class Rectangle(val width: Double, val height: Double) : Shape {
    override fun area(): Double {
        return width * height
    }

    override fun perimeter(): Double {
        return 2 * (width + height)
    }
}

// Square is a rectangle with equal sides.
class Square(val side: Double) : Rectangle(side, side)

// Returns the sum of areas for all shapes.
fun totalArea(shapes: List<Shape>): Double {
    return shapes.sumOf { it.area() }
}
