<?php

class Math {
    public function add($a, $b) {
        return $a + $b;
    }

    public function twice($n) {
        return $this->add($n, $n);
    }
}
