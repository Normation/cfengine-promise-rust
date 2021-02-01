# CFEngine Custom Promises in Rust

This library is a Rust implementation of the
[custom promise protocol](https://github.com/cfengine/core/blob/master/docs/custom_promise_types/modules.md),
added in CFEngine 3.17.

It uses the JSON variant of the protocol, and allows easily implementing promise types in
Rust with a type-safe and idiomatic interface.

## Design

Design is inspired by the [reference Python and shell implementations](https://github.com/cfengine/core/blob/master/docs/custom_promise_types).

The main goal is to provide a reliable interface, by checking as much stuff as we can
(including parameters types, etc) to allow easily implementing safe and fast promise types.
Note that we do not try to stick to close to the underlying protocol, and prefer a
an idiomatic way when possible.

This lib is done with Rudder use cases in mind, so we have a special focus on the audit mode (warn only).
In this order, we split the *evaluate* step into *check* and *apply*
to handle warn-only mode at executor level and avoid having to implement it in every promise.

The library is generally build around a trait describing a promise type's interface, and an executor
that handles the stdin/stdout communication and protocol serialization.

## Usage

To use this library add:

```toml
cfengine_promise = { git = "https://github.com/Normation/cfengine-promise-rust" }
```

To your `Cargo.toml`.

Read the `/examples` for simple promise types implementations.

## Why you should not use it

* This lib is currently in beta state
* Rust is not a very good scripting language, so if you're mostly executing commands it will be tedious
* Rust is not as portable as Unix shell or Python

## Why you should use it

* If you need performance similar to the native promise types
* Want to implement complex logic behind your promise type
