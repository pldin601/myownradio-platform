<?php

use Facebook\FacebookSession;
use Framework\Router;
use Framework\Template;

// Allow only https access
if(!isset($_SERVER['HTTPS']) || $_SERVER['HTTPS'] == ""){
    $redirect = "https://".$_SERVER['HTTP_HOST'].$_SERVER['REQUEST_URI'];
    header("HTTP/1.1 301 Moved Permanently");
    header("Location: $redirect");
    die();
}


require_once "application/startup.php";
require_once "application/libs/functions.php";
require_once "application/libs/acResizeImage.php";
require_once "dependencies/getid3/getid3.php";
require_once "dependencies/Twig/Autoloader.php";

gc_enable();

Twig_Autoloader::register(true);
FacebookSession::setDefaultApplication('731742683610572', 'f84af1cdddcc6ac06c6ebf606fb616c3');
Template::setTemplatePath("application/tmpl");

$router = Router::getInstance();

$router->route();

