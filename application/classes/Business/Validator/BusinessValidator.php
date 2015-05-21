<?php
/**
 * Created by PhpStorm.
 * User: Roman
 * Date: 14.05.15
 * Time: 15:37
 */

namespace Business\Validator;


use Framework\Services\DB\DBQuery;
use Framework\Services\ValidatorTemplates;

class BusinessValidator extends Validator {

    use ValidatorTemplates;

    const PERMALINK_REGEXP_PATTERN = "~(^[a-z0-9\\-]+$)~";
    const LOGIN_PATTERN = "~^[0-9a-z\\_]+$~";

    const LOGIN_MIN_LENGTH = 3;
    const LOGIN_MAX_LENGTH = 32;

    const PASSWORD_MIN_LENGTH = 6;
    const PASSWORD_MAX_LENGTH = 32;

    public function login() {
        $copy = $this->copy();
        $copy->addPredicate(function ($value) { return preg_match(self::LOGIN_PATTERN, $value); });
        return $copy;
    }

    public function isPermalink() {
        $copy = $this->copy();
        $copy->addPredicate(function ($value) { return is_null($value) || preg_match(self::PERMALINK_REGEXP_PATTERN, $value); });
        return $copy;
    }

    /**
     * @param int|null $user_id
     * @return $this
     */
    public function isPermalinkAvailableForUser($user_id = null) {
        $copy = $this->copy();
        $copy->addPredicate(function ($value) use ($user_id) {

            $dbq = DBQuery::getInstance();

            $query = $dbq->selectFrom("r_users")
                ->where("(permalink = :key OR uid = :key)", [":key" => $value]);

            if ($user_id) {
                $query->where("(uid != :ignore)", [":ignore" => $user_id]);
            }

            return count($query) == 0;

        });

        return $copy;
    }

    /**
     * @param int|null $stream_id
     * @return $this
     */
    public function isPermalinkAvailableForStream($stream_id = null) {
        $copy = $this->copy();
        $copy->addPredicate(function ($value) use ($stream_id) {

            $dbq = DBQuery::getInstance();

            $query = $dbq->selectFrom("r_streams")
                ->where("(permalink = :key OR sid = :key)", [":key" => $value]);

            if ($stream_id) {
                $query->where("(sid != :ignore)", [":ignore" => $stream_id]);
            }

            return count($query) == 0;

        });

        return $copy;
    }

    /**
     * @return $this
     */
    public function isEmailAvailable() {

        $copy = $this->copy();
        $copy->addPredicate(function ($value) {
            $query = DBQuery::getInstance()->selectFrom("r_users")->where("mail", $value);
            return count($query) == 0;
        });

        return $copy;

    }

    /**
     * @return $this
     */
    public function isLoginAvailable() {

        $copy = $this->copy();
        $copy->addPredicate(function ($value) {
            return count(DBQuery::getInstance()->selectFrom("r_users")->where("login", $value)) == 0;
        });

        return $copy;

    }

    public function isPasswordCorrect($hash) {
        $copy = $this->copy();
        $copy->addPredicate(function ($value) use ($hash) {
            return ;
        });

        return $copy;
    }

} 