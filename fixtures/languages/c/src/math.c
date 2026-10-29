typedef struct { double radius; } Circle;
double area(Circle c) { return 3.14 * c.radius * c.radius; }
int twice(int n) { return n * 2; }
int main() { Circle c = {2}; return twice((int)area(c)); }
