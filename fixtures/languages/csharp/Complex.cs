interface IShape { double Area(); }
class Circle : IShape { public double R; public Circle(double r){R=r;} public double Area()=>Math.PI*R*R; }
class Complex { public static double Total(IShape[] s){double t=0;foreach(var x in s)t+=x.Area();return t;} }
