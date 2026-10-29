protocol Shape { func area() -> Double }
struct Circle: Shape { let r: Double; func area() -> Double { .pi * r * r } }
func total(_ shapes: [Shape]) -> Double { shapes.reduce(0) { $0 + $1.area() } }
