<?php
/**
 * Created by PhpStorm.
 * UserModel: Roman
 * Date: 17.12.14
 * Time: 10:08
 */

namespace Framework\Services\DB\Query;


use Framework\Services\Database;
use PDO;
use Tools\Optional\Option;

abstract class BaseQuery {

    protected $tableName;

    protected $orders = [];

    protected $parameters = [
        "SET" => [],
        "WHERE" => [],
        "INSERT" => [],
        "UPDATE" => [],
        "HAVING" => []
    ];

    protected $limit = null;
    protected $offset = null;

    protected function quoteColumnName($column) {
        return "`" . $column . "`";
    }

    protected function repeat($char, $count, $glue = "") {
        $chars = [];
        for ($i = 0; $i < $count; $i++) {
            $chars[] = $char;
        }
        return implode($glue, $chars);
    }

    protected function quote(PDO $pdo, array $values) {
        $result = [];
        foreach ($values as $value) {
            $result[] = $pdo->quote($value);
        }
        return $result;
    }

    public function getParameters() {
        return array_merge($this->parameters["INSERT"], $this->parameters["SET"],
            $this->parameters["WHERE"], $this->parameters["HAVING"]);
    }

    public function orderBy($column) {
        if ($column === null) {
            $this->orders = [];
        } else {
            $this->orders[] = $column;
        }
        return $this;
    }

    public function buildLimits() {

        if (is_numeric($this->limit) && is_null($this->offset)) {
            return "LIMIT " . $this->limit;
        } else if (is_numeric($this->limit) && is_numeric($this->offset)) {
            return "LIMIT " . $this->offset . "," . $this->limit;
        } else {
            return "";
        }

    }


    protected function buildOrderBy() {

        if (count($this->orders) > 0) {
            return "ORDER BY " . implode(", ", $this->orders);
        } else {
            return "";
        }

    }

    /* Fetchers shortcuts */

    /**
     * @return Option
     */
    public function fetchOneRow() {
        return Database::doInConnection(function (Database $db) {
            /** @var SelectQuery $query */
            $query = clone $this;
            $query->limit(1);
            return $db->fetchOneRow($query);
        });
    }

    /**
     * @param int $column
     * @return Option
     */
    public function fetchOneColumn($column = 0) {
        return Database::doInConnection(function (Database $db) use (&$column) {
            $query = clone $this;
            $query->limit(1);
            return $db->fetchOneColumn($query, null, $column);
        });
    }

    /**
     * @param string|null $key
     * @param callable $callback
     * @param bool $cached
     * @return array
     */
    public function fetchAll($key = null, callable $callback = null, $cached = false) {
        return Database::doInConnection(function (Database $db) use (&$key, &$callback, &$cached) {
            return $db->fetchAll($this, null, $key, $callback, $cached);
        });
    }

    /**
     * @param $className
     * @param array $ctor_args
     * @return Option
     */
    public function fetchObject($className, array $ctor_args = []) {
        return Database::doInConnection(function (Database $db) use (&$className, &$ctor_args) {
            $query = clone $this;
            $query->limit(1);
            return $db->fetchOneObject($query, null, $className, $ctor_args);
        });
    }

    /**
     * @param $className
     * @param array $ctor_args
     * @return Object[]
     */
    public function fetchAllObjects($className, array $ctor_args = []) {
        return Database::doInConnection(function (Database $db) use (&$className, &$ctor_args) {
            return $db->fetchAllObjects($this, null, $className, $ctor_args);
        });
    }

    /**
     * @param callable $callable
     */
    public function eachRow(callable $callable) {
        Database::doInConnection(function (Database $db) use (&$callable) {
            //$db->beginTransaction();
            $db->eachRow($this, null, $callable);
            //$db->commit();
        });
    }

    /**
     * @return mixed
     */
    public function update() {
        return Database::doInConnection(function (Database $db) {
            return $db->executeUpdate($this, null);
        });
    }

    /**
     * @return mixed
     */
    public function executeInsert() {
        return Database::doInConnection(function (Database $db) {
            return $db->executeInsert($this, null);
        });
    }


} 