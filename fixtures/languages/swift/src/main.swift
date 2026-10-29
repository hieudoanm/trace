import Foundation

let shapes: [any Shape] = [
    Circle(radius: 2.0),
    Rectangle(width: 3.0, height: 4.0),
]

let total = shapes.reduce(0.0) { $0 + $1.area() }
print("Total area: \(formatFloat(total))")
