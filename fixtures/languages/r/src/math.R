Shape <- setRefClass("Shape")
Circle <- setRefClass("Circle", contains = "Shape",
  fields = list(radius = "numeric"),
  methods = list(area = function() pi * radius * radius))
twice <- function(n) n * 2
