interface Shape { double area(); }
class Circle implements Shape { double r; Circle(double r){this.r=r;} public double area(){return Math.PI*r*r;} }
class Complex { static double total(Shape[] s){double t=0;for(Shape x:s)t+=x.area();return t;} }
