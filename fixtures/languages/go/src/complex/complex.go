package complex
type Shape interface { Area() float64 }
type Circle struct { R float64 }
func (c Circle) Area() float64 { return 3.14 * c.R * c.R }
func Total(shapes []Shape) float64 { var t float64; for _,s:=range shapes { t+=s.Area() }; return t }
