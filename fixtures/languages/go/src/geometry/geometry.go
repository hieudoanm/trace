package geometry

import "math"

// Shape is an interface for geometric shapes.
type Shape interface {
    Area() float64
    Perimeter() float64
}

// Circle represents a circle with a given radius.
type Circle struct {
    Radius float64
}

// Area returns the area of the circle.
func (c Circle) Area() float64 {
    return math.Pi * c.Radius * c.Radius
}

// Perimeter returns the perimeter of the circle.
func (c Circle) Perimeter() float64 {
    return 2 * math.Pi * c.Radius
}

// Rectangle represents a rectangle with width and height.
type Rectangle struct {
    Width  float64
    Height float64
}

// Area returns the area of the rectangle.
func (r Rectangle) Area() float64 {
    return r.Width * r.Height
}

// Perimeter returns the perimeter of the rectangle.
func (r Rectangle) Perimeter() float64 {
    return 2 * (r.Width + r.Height)
}

// TotalArea returns the sum of areas for all shapes.
func TotalArea(shapes []Shape) float64 {
    total := 0.0
    for _, shape := range shapes {
        total += shape.Area()
    }
    return total
}
