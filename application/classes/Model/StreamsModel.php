<?php
/**
 * Created by PhpStorm.
 * User: roman
 * Date: 19.12.14
 * Time: 20:34
 */

namespace Model;


use MVC\Services\Database;
use MVC\Services\Injectable;
use MVC\Services\InputValidator;
use Objects\Stream;
use REST\Streams;
use Tools\Common;
use Tools\Optional;
use Tools\Singleton;

class StreamsModel {

    use Singleton, Injectable;

    /** @var UserModel $user  */

    protected $user;

    function __construct() {
        $this->user = AuthUserModel::getInstance();
    }


    public function create($name, $info, $hashtags, $category, Optional $permalink) {

        $validator = InputValidator::getInstance();

        // Validate parameters
        $validator->validateStreamName($name);
        $validator->validateStreamPermalink($permalink->get());

        $stream = new Stream();
        $stream->setUserID($this->user->getID());
        $stream->setName($name);
        $stream->setInfo($info);
        $stream->setHashTags($hashtags);
        $stream->setCategory($category);
        $stream->setPermalink($permalink->getOrElse($this->generatePermalink($name)));
        $stream->setCreated(time());

        $stream->save();

        return Streams::getInstance()->getOneStream($stream->getID());

    }

    public function generatePermalink($name) {

        $permalink = Common::toAscii($name);

        Database::doInConnection(function (Database $db) use (&$permalink) {

            while ($db->fetchOneColumn("SELECT COUNT(*) FROM r_streams WHERE permalink = ?", [$permalink])->get() !== 0) {
                if (preg_match("~^(.+)\\-(\\d+)$~m", $permalink, $matches)) {
                    $matches[2]++;
                    $permalink = sprintf("%s-%d", $matches[1], $matches[2]);
                } else {
                    $permalink .= "-1";
                }
            }

        });

        return $permalink;

    }

} 