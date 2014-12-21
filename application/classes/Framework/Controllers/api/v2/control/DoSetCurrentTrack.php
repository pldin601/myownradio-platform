<?php
/**
 * Created by PhpStorm.
 * UserModel: roman
 * Date: 15.12.14
 * Time: 20:47
 */

namespace Framework\Controllers\api\v2\control;


use Framework\Controller;
use Framework\Exceptions\ControllerException;
use Framework\Services\HttpPost;
use Model\PlaylistModel;

class DoSetCurrentTrack implements Controller {

    public function doPost(HttpPost $post) {

        $id     = $post->getParameter("id")->getOrElseThrow(ControllerException::noArgument("id"));
        $track  = $post->getParameter("track")->getOrElseThrow(ControllerException::noArgument("track"));

        PlaylistModel::getInstance($id)->scPlayByUniqueID($track);

    }

} 