<?php
/**
 * Created by PhpStorm.
 * UserModel: roman
 * Date: 13.12.14
 * Time: 18:54
 */

namespace Framework\Services;


use Framework\Injector\Injectable;
use Framework\Object;
use Framework\Services\Locale\I18n;
use Framework\View\Errors\View400Exception;
use Tools\Optional;
use Tools\Singleton;

class HttpRequest implements Injectable {

    use Singleton, Object;

    function __construct() {
    }

    /**
     * @param string $key
     * @return Optional
     */
    public function getHeader($key) {
        return Optional::ofNullable(http\Env::getRequestHeader($key));
    }

    /**
     * @return string
     */
    public function getMethod() {
        return $this->filterInputServer("REQUEST_METHOD");
    }

    /**
     * @return mixed
     */
    public function getServerAddress() {
        return $this->filterInputServer("SERVER_ADDR");
    }

    /**
     * @return mixed
     */
    public function getServerName() {
        return $this->filterInputServer("SERVER_NAME");
    }

    /**
     * @return mixed
     */
    public function getServerProtocol() {
        return $this->filterInputServer("SERVER_PROTOCOL");
    }

    /**
     * @return mixed
     */
    public function getRequestTime() {
        return $this->filterInputServer("REQUEST_TIME");
    }

    /**
     * @return Optional
     */
    public function getLanguage() {
        return Optional::ofDeceptive(substr(@filter_input(INPUT_SERVER, 'HTTP_ACCEPT_LANGUAGE'), 0, 2));
    }

    /**
     * @return mixed
     */
    public function getQueryString() {
        return $this->filterInputServer("QUERY_STRING");
    }

    /**
     * @return Optional
     */
    public function getHttpAccept() {
        return Optional::ofNullable($this->filterInputServer("HTTP_ACCEPT"));
    }

    /**
     * @return Optional
     */
    public function getHttpHost() {
        return Optional::ofNullable($this->filterInputServer("HTTP_HOST"));
    }

    /**
     * @return Optional
     */
    public function getHttpReferer() {
        return Optional::ofNullable($this->filterInputServer("HTTP_REFERER"));
    }

    /**
     * @return Optional
     */
    public function getHttpUserAgent() {
        return Optional::ofNullable($this->filterInputServer("HTTP_USER_AGENT"));
    }

    /**
     * @return Optional
     */
    public function getHttps() {
        return Optional::ofNullable($this->filterInputServer("HTTPS"));
    }

    /**
     * @return mixed
     */
    public function getRemoteAddress() {
        return $this->filterInputServer("HTTP_X_REAL_IP")
            ? $this->filterInputServer("HTTP_X_REAL_IP")
            : $this->filterInputServer("REMOTE_ADDR");
    }

    /**
     * @return mixed
     */
    public function getRemotePort() {
        return $this->filterInputServer("REMOTE_PORT");
    }

    /**
     * @return mixed
     */
    public function getRequestUri() {
        return $this->filterInputServer("REQUEST_URI");
    }

    /**
     * @return HttpGet
     */
    public function getParameters() {
        return HttpGet::getInstance();
    }

    /**
     * @return HttpPost
     */
    public function getPost() {
        return HttpPost::getInstance();
    }

    /**
     * @param string $parameter
     * @return Optional
     */
    public function getParameter($parameter) {
        $sources = [
            HttpPost::getInstance()->getParameter($parameter),
            HttpPut::getInstance()->getParameter($parameter),
            HttpGet::getInstance()->getParameter($parameter)
        ];
        return array_first(
            array_map(function (Optional $o) { return $o->get(); },
                array_filter($sources, function (Optional $source) {
                    return $source->notEmpty();
                })
            )
        );
    }

    /**
     * @param $parameter
     * @return mixed
     */
    public function getParameterOrFail($parameter) {
        return $this->getParameter($parameter)
            ->getOrElseThrow($this->getException($parameter));
    }

    /**
     * @param string $param
     * @return mixed
     */
    private function filterInputServer($param) {
        return FILTER_INPUT(INPUT_SERVER, $param);
    }

    /**
     * @param $key
     * @return View400Exception
     */
    private function getException($key) {
        return new View400Exception(I18n::tr("ERROR_NO_ARGUMENT_SPECIFIED", [ $key ]));
    }
} 