class Shape { double area() => 0; }
class Circle implements Shape {
  final double radius;
  Circle(this.radius);
  double area() => 3.14 * radius * radius;
}
int twice(int n) => n * 2;
void main() { var c = Circle(2); print(twice(c.area().toInt())); }
