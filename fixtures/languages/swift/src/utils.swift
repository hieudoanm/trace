import Foundation

// Utility formatting functions.
func formatFloat(_ value: Double) -> String {
    return String(format: "%.2f", value)
}

func circleArea(radius: Double) -> Double {
    return .pi * radius * radius
}
