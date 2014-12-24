<?php
/**
 * Created by PhpStorm.
 * UserModel: Roman
 * Date: 16.12.14
 * Time: 14:40
 */

namespace Framework\Controllers\api\v2\track;


use Framework\Controller;
use Framework\Models\TracksModel;
use Framework\Services\HttpFiles;
use Framework\Services\HttpPost;
use Framework\Services\JsonResponse;

class DoUpload implements Controller {

    public function doPost(HttpFiles $file, HttpPost $post, TracksModel $model, JsonResponse $response) {

        $streamID = $post->getParameter("track_id");

        $file->each(function ($file) use ($streamID, $model) {
            if (is_array($file["name"])) {
                for ($i = 0; $i < count($file["name"]); $i++) {
                    $tmp = [
                        "name" => $file["name"][$i],
                        "type" => $file["type"][$i],
                        "tmp_name" => $file["tmp_name"][$i],
                        "error" => $file["error"][$i],
                        "size" => $file["size"][$i]
                    ];
                    $model->upload($tmp, $streamID);
                }
            } else {
                $model->upload($file, $streamID);
            }
        });

    }

} 