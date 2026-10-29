from typing import Protocol
class Shape(Protocol):
    def area(self) -> float: ...
class Circle(Shape):
    def __init__(self, r: float): self.r = r
    def area(self) -> float: return 3.14159 * self.r * self.r
def total(shapes): return sum(s.area() for s in shapes)
