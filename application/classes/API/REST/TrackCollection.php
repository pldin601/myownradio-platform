<?php
/**
 * Created by PhpStorm.
 * User: roman
 * Date: 05.04.15
 * Time: 13:27
 */

namespace API\REST;


use Framework\Exceptions\ControllerException;
use Framework\Injector\Injectable;
use Framework\Models\AuthUserModel;
use Framework\Services\DB\Query\SelectQuery;
use Tools\Singleton;
use Tools\SingletonInterface;
use Tools\System;

class TrackCollection implements Injectable, SingletonInterface {
    use Singleton;

    const TRACKS_PER_REQUEST_MAX = 50;

    /**
     * @return SelectQuery
     */
    private function getTracksPrefix() {

        $prefix = (new SelectQuery("r_tracks"))
            ->innerJoin("mor_track_stat", "mor_track_stat.track_id = r_tracks.tid")
            ->select("r_tracks.tid", "r_tracks.filename", "r_tracks.artist", "r_tracks.title", "r_tracks.album",
                "r_tracks.track_number", "r_tracks.genre", "r_tracks.date", "r_tracks.buy", "r_tracks.duration",
                "r_tracks.color", "r_tracks.can_be_shared", "mor_track_stat.likes", "mor_track_stat.dislikes");

        return $prefix;

    }

    /**
     * @return SelectQuery
     */
    private function getChannelQueuePrefix() {

        $prefix = $this->getTracksPrefix();
        $prefix->innerJoin("r_link", "r_link.track_id = r_tracks.tid");
        $prefix->select("r_link.t_order", "r_link.unique_id", "r_link.time_offset");

        return $prefix;

    }

    /**
     * @return SelectQuery
     */
    private function getSchedulePrefix() {

        $prefix = $this->getChannelQueuePrefix();

        $prefix->innerJoin("r_streams", "r_link.stream_id = r_streams.sid");
        $prefix->innerJoin("r_static_stream_vars", "r_streams.sid = r_static_stream_vars.stream_id");

        $prefix->where("r_link.time_offset <= MOD(:micro - (r_streams.started - r_streams.started_from), r_static_stream_vars.tracks_duration)", [ ":micro" => System::time() ]);
        $prefix->where("r_link.time_offset + r_tracks.duration > MOD(:micro - (r_streams.started - r_streams.started_from), r_static_stream_vars.tracks_duration)");
        $prefix->where("r_streams.status = 1 AND r_static_stream_vars.tracks_duration > 0");

        $prefix->select("r_streams.sid");

        return $prefix;

    }

    /**
     * @param int $channel_id
     * @return array
     */
    public function getPlayingOnChannel($channel_id) {

        $query = $this->getSchedulePrefix();

        $query->select("r_static_stream_vars.listeners_count");
        $query->select("r_static_stream_vars.bookmarks_count");

        $query->where("r_streams.sid", $channel_id);

        $query->select(":micro AS time");
        $query->select("MOD(:micro - (r_streams.started - r_streams.started_from), r_static_stream_vars.tracks_duration) AS position");

        return $query->fetchOneRow()->getOrElseThrow(ControllerException::of("NO_TRACK_PLAYING"));

    }

    /**
     * @param array $channel_ids
     * @return array
     */
    public function getPlayingOnChannels(array $channel_ids) {

        $query = $this->getSchedulePrefix();

        $query->select("r_static_stream_vars.listeners_count");
        $query->select("r_static_stream_vars.bookmarks_count");

        $query->where("r_streams.sid", $channel_ids);

        return $query->fetchAll("sid");

    }

    /**
     * @param int $offset
     * @param int $limit
     * @internal UserModel $self
     * @return array
     */
    public function getTracksFromLibrary($offset = 0, $limit = self::TRACKS_PER_REQUEST_MAX) {

        $query = $this->getTracksPrefix();
        $self = AuthUserModel::getInstance();

        $query->where("r_tracks.uid", $self->getID());

        $query->offset($offset);
        $query->limit($limit);

        $query->orderBy("r_tracks.uploaded DESC");

        return $query->fetchAll();

    }

    /**
     * @param $channel_id
     * @param int $offset
     * @param int $limit
     * @internal UserModel $self
     * @return array
     */
    public function getTracksFromChannel($channel_id, $offset = 0, $limit = self::TRACKS_PER_REQUEST_MAX) {

        $query = $this->getChannelQueuePrefix();
        $self = AuthUserModel::getInstance();

        $query->where("r_link.stream_id", $channel_id);
        $query->where("r_tracks.uid", $self->getID());

        $query->orderBy("r_link.t_order ASC");

        $query->offset($offset);
        $query->limit($limit);

        return $query->fetchAll();

    }

    /**
     * @param $track_id
     * @return mixed
     */
    public function getSingleTrack($track_id) {

        $query = $this->getTracksPrefix();

        $query->where("r_tracks.tid", $track_id);

        return $query->fetchOneRow()->getOrElseThrow(ControllerException::noTrack($track_id));

    }

} 