package shapes

// Shape is a protocol for geometric shapes.
protocol Shape {
    func area() -> Double
    func perimeter() -> Double
}

// Circle represents a circle with a given radius.
struct Circle: Shape {
    var radius: Double

    init(radius: Double) {
        self.radius = radius
    }

    func area() -> Double {
        return .pi * radius * radius
    }

    func perimeter() -> Double {
        return 2 * .pi * radius
    }
}

// Rectangle represents a rectangle with width and height.
struct Rectangle: Shape {
    var width: Double
    var height: Double

    init(width: Double, height: Double) {
        self.width = width
        self.height = height
    }

    func area() -> Double {
        return width * height
    }

    func perimeter() -> Double {
        return 2 * (width + height)
    }
}

// Square is a rectangle with equal sides.
struct Square: Shape {
    var side: Double

    init(side: Double) {
        self.side = side
    }

    func area() -> Double {
        return side * side
    }

    func perimeter() -> Double {
        return 4 * side
    }
}

// totalArea returns the sum of areas for all shapes.
func totalArea(shapes: [Shape]) -> Double {
    return shapes.reduce(0.0) { $0 + $1.area() }
}
