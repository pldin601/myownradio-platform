<?php
/**
 * Created by PhpStorm.
 * User: roman
 * Date: 13.12.14
 * Time: 20:42
 */

namespace MVC\Exceptions;

class ControllerException extends \Exception {

    private $myMessage = null;
    private $myData = [];

    function __construct($message = null, $data = null) {
        $this->myMessage = $message;
        $this->myData = $data;
    }

    public function getMyData() {
        return $this->myData;
    }

    public function getMyMessage() {
        return $this->myMessage;
    }

    public static function noArgument($name) {
        return new self(sprintf("No value for argument '%s' specified", $name));
    }
} 