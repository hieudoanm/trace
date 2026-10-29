"""Geometric shape classes."""

import math


class Circle:
    """A circle with a given radius."""

    def __init__(self, radius: float) -> None:
        self.radius = radius

    def area(self) -> float:
        """Return the area of the circle."""
        return math.pi * self.radius * self.radius

    def perimeter(self) -> float:
        """Return the perimeter of the circle."""
        return 2 * math.pi * self.radius


class Rectangle:
    """A rectangle with width and height."""

    def __init__(self, width: float, height: float) -> None:
        self.width = width
        self.height = height

    def area(self) -> float:
        """Return the area of the rectangle."""
        return self.width * self.height

    def perimeter(self) -> float:
        """Return the perimeter of the rectangle."""
        return 2 * (self.width + self.height)


class Square(Rectangle):
    """A square is a rectangle with equal sides."""

    def __init__(self, side: float) -> None:
        super().__init__(side, side)
