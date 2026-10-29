package main

import (
	"fmt"

	example.com/geometry
)

func main() {
	shapes := []geometry.Shape{
		geometry.Circle{Radius: 2.0},
		geometry.Rectangle{Width: 3.0, Height: 4.0},
	}
	fmt.Printf("Total area: %.2f\n", geometry.TotalArea(shapes))
}
