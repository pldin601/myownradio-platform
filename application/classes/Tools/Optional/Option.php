<?php
/**
 * Created by PhpStorm.
 * User: Roman
 * Date: 26.06.2015
 * Time: 13:29
 */

namespace Tools\Optional;

/**
 * Class Option
 * @package Tools\Optional
 */
abstract class Option implements \IteratorAggregate, \JsonSerializable {

    use OptionMixin;

    /**
     * @return bool
     */
    public abstract function isEmpty();

    /**
     * @return mixed
     */
    public abstract function get();

    /**
     * @return \Iterator
     */
    public abstract function getIterator();

    /**
     * @return bool
     */
    public abstract function nonEmpty();

    /**
     * @param $other
     * @return mixed
     */
    public abstract function getOrElse($other);

    /**
     * @return mixed
     */
    public abstract function orFalse();

    /**
     * @return mixed
     */
    public abstract function orZero();

    /**
     * @return mixed
     */
    public abstract function orNull();

    /**
     * @return mixed
     */
    public abstract function orEmpty();

    /**
     * @param $callable
     * @return mixed
     */
    public abstract function orCall($callable);

    /**
     * @param Option $alternative
     * @return Option
     */
    public abstract function orElse(Option $alternative);

    /**
     * @param $exception
     * @param ...$args
     * @return mixed
     */
    public abstract function getOrThrow($exception, ...$args);

    /**
     * @param $callable
     * @param $args
     * @return $this
     */
    public abstract function map($callable, ...$args);

    /**
     * @param $callable
     * @return $this
     */
    public abstract function flatMap($callable);

    /**
     * @param $predicate
     * @return $this
     */
    public abstract function filter($predicate);

    /**
     * @param $predicate
     * @return $this
     */
    public abstract function reject($predicate);

    /**
     * @param $callable
     * @param ...$args
     * @return $this
     */
    public abstract function orThrow($callable, ...$args);

    /**
     * @param callable $callable
     * @param callable|null $otherwise
     * @return $this
     */
    public abstract function then($callable, $otherwise = null);

    /**
     * @param \Closure $producer
     * @return $this
     */
    public abstract function otherwise(\Closure $producer);

    /**
     * @param $value
     * @return $this
     */
    public abstract function select($value);

    /**
     * @param $object
     * @return $this
     */
    public abstract function selectInstance($object);

    /**
     * @return Option
     */
    public static function None() {
        return None::instance();
    }

    /**
     * @param $value
     * @return Option
     */
    public static function Some($value) {
        return new Some($value);
    }


    function jsonSerialize() {
        return $this->get();
    }

}


