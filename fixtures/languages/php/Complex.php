<?php
interface Shape { public function area(): float; }
class Circle implements Shape { public function __construct(public float $r) {} public function area(): float { return pi()*$this->r*$this->r; } }
function total(array $s): float { $t=0; foreach($s as $x)$t+=$x->area(); return $t; }
