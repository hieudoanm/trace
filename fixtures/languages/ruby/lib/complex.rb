module Shape; def area; end; end
class Circle; include Shape; attr_reader :r; def initialize(r); @r=r; end; def area; Math::PI*@r*@r; end; end
def total(shapes); shapes.map(&:area).sum; end
