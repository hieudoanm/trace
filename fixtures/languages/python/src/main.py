"""Geometry utilities for computing areas and perimeters."""

from __future__ import annotations

import math
from typing import Protocol

from shapes import Circle, Rectangle
from utils import format_float


class Shape(Protocol):
    """Protocol for shapes with an area."""

    def area(self) -> float:
        ...


def total_area(shapes: list[Shape]) -> float:
    """Return the sum of areas for all shapes."""
    return sum(shape.area() for shape in shapes)


def main() -> None:
    shapes: list[Shape] = [Circle(2.0), Rectangle(3.0, 4.0)]
    print(f"Total area: {format_float(total_area(shapes))}")


if __name__ == "__main__":
    main()
