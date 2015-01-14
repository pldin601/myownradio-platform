<?php
/**
 * Created by PhpStorm.
 * UserModel: roman
 * Date: 13.12.14
 * Time: 20:42
 */

namespace Framework\Exceptions;

class ControllerException extends \Exception {

    private $myMessage = null;
    private $myData = [];

    function __construct($message = null, $data = null) {
        $this->myMessage = $message;
        $this->myData = $data;
    }

    public static function of($message = null, $data = null) {
        return new self($message, $data);
    }

    public static function noBasis($id) {
        return new self(sprintf("No payment basis with key '%s' found", $id));
    }

    public static function noImageAttached() {
        return new self("No image file attached");
    }

    public function getMyData() {
        return $this->myData;
    }

    public function getMyMessage() {
        return $this->myMessage;
    }

    public static function wrongLogin() {
        return new self("Incorrect login or password");
    }

    public static function noArgument($name) {
        return new self(sprintf("No value for argument '%s' specified", $name));
    }

    public static function databaseError($message = "Something wrong with database") {
        return new self($message);
    }

    public static function noStream($key) {
        return new self(sprintf("No stream with key '%s' found", $key));
    }

    public static function noPermission() {
        return new self("You don't have permission to access this resource");
    }

    public static function noEntity($name) {
        return new self(sprintf("No entity '%s' found", $name));
    }

    public static function noTrack($key) {
        return new self(sprintf("No track with key '%s' found", $key));
    }


}