<?php
/**
 * Created by PhpStorm.
 * UserModel: Roman
 * Date: 15.12.14
 * Time: 10:05
 */

namespace MVC\Controllers\api\v2\self;


use Model\UsersModel;
use MVC\Controller;

class DoLogout extends Controller {

    public function doPost() {
        UsersModel::unAuthorize();
    }

} 