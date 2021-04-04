<?php
/**
 * Created by PhpStorm.
 * UserModel: Roman
 * Date: 17.12.14
 * Time: 13:02
 */

namespace Framework\Services\DB\Query;


use PDO;

trait WhereSection {

    protected $wheres = [];
    protected $whereGlue = "AND";

    // Where section

    public function where($clause) {
        if (func_num_args() == 2 && is_array(func_get_arg(1))) {
            $this->whereArray(func_get_arg(0), func_get_arg(1));
        } elseif (func_num_args() == 1 && is_array(func_get_arg(0))) {
            $this->whereHashMap(func_get_arg(0));
        } elseif (func_num_args() == 2) {
            $this->whereSimple(func_get_arg(0), func_get_arg(1));
        } elseif (func_num_args() == 1) {
            $this->whereSimple("(".$clause.")");
        }
        return $this;
    }

    /**
     * @param $column
     * @param $values
     * @return $this
     */
    public function whereFindInSet($column, $values) {
        $this->wheres[] = "FIND_IN_SET(" . $column . ", ?)";
        $this->parameters["WHERE"][] = $values;
        return $this;
    }


    private function whereSimple($column, $value = null) {
        if (is_null($value)) {
            $this->wheres[] = $column;
        } else {
            $this->parameters["WHERE"][] = $value;
            $this->wheres[] = [$column, "?"];
        }
    }

    private function whereParameters($clause, array $parameters) {
        foreach ($parameters as $key => $parameter) {
            if (is_numeric($key)) {
                $this->parameters["WHERE"][] = $parameter;
            } else {
                $this->parameters["WHERE"][$key] = $parameter;
            }
        }
        $this->wheres[] = $clause;
    }

    private function whereArray($column, array $values) {
        if (preg_match("~(\\?)|(\\:\\w+)~m", $column)) {
            $this->whereParameters($column, $values);
        } else {
            $this->wheres[] = [$column, $values];
        }
    }

    private function whereHashMap(array $map) {
        foreach ($map as $key => $value) {
            $this->whereSimple($key, $value);
        }
    }


    private function buildWheres(PDO $pdo) {

        $build = [];

        foreach ($this->wheres as $where) {
            if (is_string($where)) {
                $build[] = $where;
            } else if (count($where) == 2 && is_array($where[1])) {
                $build[] = $where[0] . " IN (" . implode(",", $this->quote($pdo, $where[1])) . ")";
            } else {
                $build[] = $where[0] . "=" . $where[1];
            }
        }

        return "WHERE " . (count($build) > 0 ? implode(" {$this->whereGlue} ", $build) : "1");

    }

    /**
     * @param string $glue
     * @return $this
     */
    public function setWhereGlue($glue) {
        $this->whereGlue = strval($glue);
        return $this;
    }


} 