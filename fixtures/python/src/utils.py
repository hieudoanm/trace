"""Utility formatting functions."""

import math


def format_float(value: float) -> str:
    """Format a float to two decimal places."""
    return f"{value:.2f}"


def circle_area(radius: float) -> float:
    """Compute the area of a circle given its radius."""
    return math.pi * radius * radius
