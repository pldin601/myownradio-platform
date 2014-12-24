<?php
/**
 * Created by PhpStorm.
 * UserModel: Roman
 * Date: 16.12.14
 * Time: 10:19
 */

namespace Framework\Controllers\api\v2\self;


use Framework\Controller;
use Framework\Exceptions\ControllerException;
use Framework\Models\UsersModel;
use Framework\Services\HttpPost;
use Framework\Services\InputValidator;
use Framework\Services\JsonResponse;

class DoSignUpComplete implements Controller {

    public function doPost(HttpPost $post, InputValidator $validator, UsersModel $users, JsonResponse $response) {

        $code = $post->getParameter("code")->getOrElseThrow(ControllerException::noArgument("code"));
        $login = $post->getParameter("login")->getOrElseThrow(ControllerException::noArgument("login"));
        $password = $post->getParameter("password")->getOrElseThrow(ControllerException::noArgument("password"));
        $name = $post->getParameter("name")->getOrElseNull();
        $info = $post->getParameter("info")->getOrElseNull();
        $permalink = $post->getParameter("permalink")->getOrElseNull();

        $validator->validateRegistrationCode($code);
        $validator->validatePassword($password);
        $validator->validateLogin($login);
        $validator->validateUserPermalink($permalink);

        $users->completeRegistration($code, $login, $password, $name, $info, $permalink);

    }

} 