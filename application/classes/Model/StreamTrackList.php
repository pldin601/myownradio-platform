<?php
/**
 * Created by PhpStorm.
 * User: Roman
 * Date: 15.12.14
 * Time: 11:49
 */

namespace Model;

use MVC\Exceptions\ControllerException;
use Tools\Optional;
use Tools\Singleton;
use Tools\System;

class StreamTrackList extends Model {

    use Singleton;

    private $key;
    private $user;

    private $tracks_count;
    private $tracks_duration;

    private $status;
    private $started_from;
    private $started;

    public function __construct($id) {
        parent::__construct();
        $this->user = new User();
        $this->key = $id;
        $this->reload();
    }

    /**
     * @throws ControllerException
     * @return $this
     */
    public function reload() {

        $stats = $this->db->fetchOneRow("SELECT a.uid,a.started, a.started_from, a.status, b.tracks_count, b.tracks_duration
            FROM r_streams a LEFT JOIN r_static_stream_vars b on a.sid = b.stream_id WHERE a.sid = ?",
            [$this->key])->getOrElseThrow(ControllerException::noStream($this->key));

        if (intval($stats["uid"]) !== $this->user->getId()) {
            throw ControllerException::noPermission();
        }

        $this->tracks_count = intval($stats["tracks_count"]);
        $this->tracks_duration = intval($stats["tracks_duration"]);

        $this->status = intval($stats["status"]);
        $this->started = intval($stats["started"]);
        $this->started_from = intval($stats["started_from"]);

        return $this;

    }

    public function getStreamPosition() {

        if ($this->tracks_duration == 0) {
            return Optional::ofNull(0);
        }

        if ($this->status == 0) {
            return Optional::ofNull(null);
        }

        $time = System::time();

        $position = ($time - $this->started + $this->started_from) % $this->tracks_duration;

        return Optional::ofNull($position);

    }

    /**
     * @return Optional
     */
    public function getCurrentTrack() {

        $time = $this->getStreamPosition();


        if ($time->validate()) {
            return $this->getTrackByTime($time->getRaw());
        }

        return Optional::bad();

    }

    /**
     * @param $time
     * @return $this
     */
    public function getTrackByTime($time) {

        $track = $this->db->fetchOneRow("

            SELECT a.*, b.unique_id, b.t_order, b.time_offset
            FROM r_tracks a LEFT JOIN r_link b ON a.tid = b.track_id
            WHERE b.time_offset <= :time AND b.time_offset + a.duration >= :time AND b.stream_id = :id

            ", [":time" => $time, ":id" => $this->key])

            ->then(function (&$track) use ($time) {
                $track['cursor'] = $time - $track['time_offset'];
            });

        return $track;

    }



    private function doAtomic(callable $callable) {

        //$position = $this->getStreamPosition()->getOrElseNull();

        $result = call_user_func($callable);

        return $result;

    }

} 