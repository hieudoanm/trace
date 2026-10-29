<?php

namespace App\Controller;

use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\Routing\Annotation\Route;

class AppController extends AbstractController
{
    #[Route('/hello', name: 'app_hello')]
    public function hello(): string
    {
        return 'Hello Symfony';
    }
}
