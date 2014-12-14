<?php

namespace Model;

use MVC\Exceptions\ControllerException;
use MVC\Exceptions\UnauthorizedException;
use MVC\Services\Injectable;
use Tools\Singleton;

class User extends Model {

    use Singleton, Injectable;

    private $userId;
    private $userLogin;
    private $userName;
    private $userEmail;
    private $userInfo;

    private $userToken;
    
    public function __construct() {

        parent::__construct();

        $uid = $this->getIdBySessionToken();
        $user = $this->db->fetchOneRow("SELECT * FROM r_users WHERE uid = ?", array($uid))
            ->getOrElseThrow(ControllerException::noPermission());

        $this->userId       = intval($user['uid']);
        $this->userLogin    = $user['login'];
        $this->userName     = $user['name'];
        $this->userEmail    = $user['mail'];
        $this->userInfo     = $user['info'];

    }
    
    public function getId()
    {
        return $this->userId;
    }
    
    public function getLogin()
    {
        return $this->userLogin;
    }
    
    public function getEmail()
    {
        return $this->userEmail;
    }

    public function getName() {
        return $this->userName;
    }

    public function getToken()
    {
        return $this->userToken;
    }

    public function changePassword($password) {
        $newPassword = md5($this->getLogin() . $password);
        $this->db->executeUpdate("UPDATE r_users SET password = ? WHERE uid = ?", array($newPassword, $this->userId));
    }

    /**
     * @param mixed $email
     * @return self
     */
    public function setUserEmail($email) {
        $this->userEmail = $email;
        return $this;
    }

    /**
     * @param mixed $name
     * @return self
     */
    public function setName($name) {
        $this->userName = $name;
        return $this;
    }

    /**
     * @param mixed $info
     * @return self
     */
    public function setInfo($info) {
        $this->userInfo = $info;
        return $this;
    }


    public function getIdBySessionToken() {
        $exception = new UnauthorizedException();
        $token = \session::get('authtoken');
        if (is_null($token)) {
            throw $exception;
        }
        return $this->db->fetchOneColumn("SELECT b.uid FROM r_sessions a LEFT JOIN r_users b ON a.uid = b.uid WHERE a.token = ?", array($token))
            ->getOrElseThrow($exception);
    }

    public function update() {
        $this->db->executeUpdate("UPDATE r_users SET name = ?, info = ?, mail = ? WHERE uid = ?",
        [$this->userName, $this->userInfo, $this->userEmail, $this->userId]);
    }

    static function createToken($userId, $userIP, $userAgent, $sessionId) {
/*        do
        {
            $token = md5($uid . $ip . rand(1,1000000) . "tokenizer" . time());
        }
        while(db::query_single_col("SELECT COUNT(*) FROM `r_sessions` WHERE `token` = ?", array($token)) > 0);
        
        db::query_update("INSERT INTO `r_sessions` SET `uid` = ?, `ip` = ?, `token` = ?, `permanent` = 1, `authorized` = NOW(), `http_user_agent` = ?, `session_id` = ?, `expires` = NOW() + INTERVAL 1 YEAR", array(
            $uid, $ip, $token, $ua, $session_id
        ));
        
        return $token;*/
    }
}
