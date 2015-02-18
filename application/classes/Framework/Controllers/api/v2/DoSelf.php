<?php
/**
 * Created by PhpStorm.
 * User: roman
 * Date: 27.12.14
 * Time: 14:02
 */

namespace Framework\Controllers\api\v2;


use Framework\Controller;
use Framework\Exceptions\ControllerException;
use Framework\Models\AuthUserModel;
use Framework\Models\UsersModel;
use Framework\Services\HttpPost;
use Framework\Services\HttpPut;
use Framework\Services\InputValidator;
use Framework\Services\JsonResponse;
use REST\Streams;
use REST\Users;

class DoSelf implements Controller {

    public function doGet(AuthUserModel $userModel, JsonResponse $response, Streams $streams, Users $users) {

        $response->setData([
            'user'      => $users->getUserByID($userModel->getID()),
            'streams'   => $streams->getByUser($userModel->getID())
        ]);
        
    }

    public function doPut(HttpPut $put, UsersModel $users, JsonResponse $response) {

        $login = $put->getRequired("login");
        $password = $put->getRequired("password");
        $remember = boolval($put->getParameter("remember")->getOrElseFalse());

        $users->logout();
        $users->authorizeByLoginPassword($login, $password);

    }

    public function doPost(HttpPost $post, AuthUserModel $user, JsonResponse $response, InputValidator $validator) {

        $name       = $post->getRequired("name");
        $info       = $post->getParameter("info")->getOrElseEmpty();
        $permalink  = $post->getParameter("permalink")->getOrElseNull();
        $countryId  = $post->getParameter("country_id")->getOrElseNull();

        $validator->validateUserPermalink($permalink, $user->getID());
        $validator->validateCountryID($countryId);

        $user->edit($name, $info, $permalink, $countryId);

    }

    public function doDelete(UsersModel $users, JsonResponse $response) {

        $users->logout();

    }

} 