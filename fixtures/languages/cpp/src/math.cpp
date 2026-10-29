class Shape { public: virtual double area() = 0; };
class Circle : public Shape {
  double radius;
public:
  Circle(double r): radius(r) {}
  double area() override { return 3.14159 * radius * radius; }
};
int twice(int n) { return n * 2; }
int main() { Circle c(2); return twice((int)c.area()); }
