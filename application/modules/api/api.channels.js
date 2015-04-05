/**
 * Created by roman on 05.04.15.
 */
(function () {

    var api = angular.module("application");

    api.service("$api", ["$q", "$http", function ($q, $http) {

        function answer(promise) {
            return $q(function (resolve, reject) {
                promise.then(function (response) {
                    if (response.data.code == 1) {
                        resolve(response.data.data);
                    } else {
                        reject(response.data.message);
                    }
                })
            });
        }

        return {
            get: function (url, params) {
                return answer($http.get(url, {
                    params: params
                }));
            },
            post: function (url, post) {
                return answer($http.post(url, post));
            },
            filter: function (arguments) {
                var obj = {};
                for (var key in arguments) if (arguments.hasOwnProperty(key)) {
                    if (typeof arguments[key] != "undefined") {
                        obj[key] = arguments[key];
                    }
                }
                return obj;
            }
        }

    }]);

    api.service("$channels", ["$api", function ($api) {
        return {
            getAllChannels: function (offset, limit) {
                return $api.get("/api/v2/channels/all", $api.filter({
                    offset: offset,
                    limit: limit
                }));
            },
            getCategoryChannels: function (category, offset, limit) {
                return $api.get("/api/v2/channels/category", $api.filter({
                    category_name: category,
                    offset: offset,
                    limit: limit
                }));
            },
            getMyChannels: function (offset, limit) {
                return $api.get("/api/v2/channels/my", $api.filter({
                    offset: offset,
                    limit: limit
                }));
            },
            getPopularChannels: function (offset, limit) {
                return $api.get("/api/v2/channels/popular", $api.filter({
                    offset: offset,
                    limit: limit
                }));
            },
            getSearchChannels: function (filter, offset, limit) {
                return $api.get("/api/v2/channels/search", $api.filter({
                    query: filter,
                    offset: offset,
                    limit: limit
                }));
            },
            getSuggestChannels: function (filter, offset, limit) {
                return $api.get("/api/v2/channels/suggest", $api.filter({
                    query: filter,
                    offset: offset,
                    limit: limit
                }));
            },
            getTagChannels: function (tag, offset, limit) {
                return $api.get("/api/v2/channels/tag", $api.filter({
                    tag: tag,
                    offset: offset,
                    limit: limit
                }));
            },
            getUserChannels: function (user, offset, limit) {
                return $api.get("/api/v2/channels/user", $api.filter({
                    key: user,
                    offset: offset,
                    limit: limit
                }));
            },
            getBookmarkedChannels: function (offset, limit) {

            }
        }
    }]);

})();