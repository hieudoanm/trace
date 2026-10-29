trait Shape { def area: Double }
class Circle(val radius: Double) extends Shape { def area = math.Pi * radius * radius }
object Math { def twice(n: Int): Int = n * 2 }
