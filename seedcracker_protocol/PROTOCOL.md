# The Minecraft Seed Cracker Interface Protocol - MCSCI

## 📄 Abstract

This document defines the specification for the **MCSCI** protocol, a line-based command and data transfer protocol designed for both human readability and efficient machine parsing. The protocol supports a core set of commands, and an extensible mechanism for defining custom commands. The primary use case is the structured exchange of commands and typed data between a client and a server, with clear acknowledgment and error reporting.

---

## 🎯 Objectives

- **Simplicity**  
  Create a protocol that is easy to understand, learn, and implement.

- **Human Readability**  
  Ensure all commands and responses are clear when read in plain text.

- **Extensibility**  
  Allow servers to use protocol extensions to expand their functionality while maintaining compatibility with clients that might not support them.

- **Machine Parseability**  
  Use unambiguous simple grammar to facilitate robust automated parsing.

- **Consistency**  
  Use predictable command and response formats.

---

## 📝 Specification Overview

The specification is divided into two main sections, which are both subject to possible extension in the future:

1. **Core Commands/Responses**
2. **Extensions**
3. **Typed Values**

This document currently defines version 0 of the protocol. This v0 version is intended for testing and development, and is not intended for use in production. It is subject to change at any time without notice, including, but not limited to, adding new commands, modifying existing commands, removing existing commands, and any kind of breaking change. The protocol should be considered in a beta state at this point, until version 1 is released. The only guarantee for version 0 is that this document reflects the current state of the rust implementation in this same repository.

The protocol is **line-oriented**, where each command is a single line terminated by `\n`. For maximum compatibility, a MCSCI-compliant client or server should support both `\r\n` and `\n` line endings.
Every non-empty line sent by the client is interpreted as a command, and every non-empty line sent by the server is interpreted as a response.
The protocol is **NOT stateless**, and the server maintains a state that is updated based on the commands and responses. This state is not directly visible to the client, but is used by the server when responding to commands.
The protocol is **deterministic**, and all commands and responses are deterministic based on the current state of the server. 
When a connection is initiated, the server creates a state that is proper to that connection, and that is destroyed when the connection is closed, regardless of the connection type (TCP, stdio, etc.). The server initially waits for the `hello` command from the client.
When the server receives a command it can't understand, parse, or execute for any reason (which include; but are not limited to, malformed commands, unknown commands, invalid command in current context, etc.), it will ignore it and respond with an error, which is specified in the [Core Commands/Responses](#core-commands--responses) section.
When a client can't understand a response, it can decide to ignore it, in which case there are absolutely no guarantees about the state of the server, or decide to terminate the connection.
Finally, when the server receives a command it can understand and execute, it must respond with the acknowledgement of the command, which is specified in the [Core Commands/Responses](#core-commands--responses) section. After the acknowledgement is sent, the client waits for the response before sending the next command, meaning the protocol is strictly synchronous, and none of the core feature can be parallelized, while extentions manage their own behaviour. The client should expect and ignore any responses that start with `info` as they are used to inform a human client, and are not relevant to an automated client. The server sends these information messages whenever it wants to, and the client should be prepared to receive them. Similarly, responses that start with `status` are used to inform the client (human or automated) of the current status of the (possibly) long-running task the server is performing, and the client should be prepared to receive them.

## Core Commands / Responses

This section defines the core commands of the protocol per versions, and their corresponding responses.
It uses a simple grammar to specify how the commands and responses should be formatted:
- When using a standalone string, it means that the exact string is expected at this place. For example: `hello` means that we expect to get exactly the string `hello`.
- When using a pipe character (`|`) in between of two or more expressions, it means that exactly one of the expressions is expected at this place. For example: `hello|quit` means that we expect to get either the string `hello` or the string `quit`, and `a|b|c` means that we expect to get either the string `a`, the string `b`, or the string `c`.
- Brackets (`[`, `]`) are used to surround expressions into a larger expression. It is used in conjunction with other operators.
- When an expression is followed by a question mark (`?`), it means that the expression is optional. For example: `hello?` means that we expect to get either the string `hello` or nothing, and `[a|b]?c` means that we expect to get optionnaly an expression of the form `a|b`, that has to be followed by the string `c`. This means `ac`, `bc` and `c` are valid.
- When an expression is followed by a plus (`+`), it means this expression is expected to be repeated **one or more** times. For example, `abc+` means "at least one `abc`", such that `abc`, `abcabc`, `abcabcabc`, ..., are valid, but not `ab` and `abca`. `a[b+]c` would mean "an `a` followed by one ore more `b`s and then a `c`", such that `abc`, `abbc`, `abbbc`, ..., are valid but `ac` is not.
- When an expression is followed by an asterisk (`*`), it means this expression is expected to be repeated **zero or more** times. For example, `abc*` means "zero or more `abc`s", such that ` `, `abc`, `abcabc`, `abcabcabc`, ..., are valid, but not `ab` and `abca`. Additionally, `a*` is logically equivalent `a+?` and `a?+`, although `a?+` causes recursion issues if parsing is implemented naively.
- Anything surrounded by `<` and `>` is a human readable description of what is expected at this place. It is a single expression. For example `<protocol version number>` is used to specify that the expression should correspond to a number, and that it describes a protocol version number.

### Version 0

Commands:

- `hello`  
  Sent by the client to initiate a connection.
  The server should respond with the `ack` acknowledgement.

- `help`  
  Lists all available commands.
  The server should respond with the `ack` acknowledgement.

- `quit`  
  Prepares the server to close the connection. If the client sends this command, the server is expected to close the connection.
  The server may not respond with the `ack` acknowledgement.
  
- `version`  
  Reports the version of the protocol.
  The server should respond with the `ack` acknowledgement, then a `version` response containing the version of the protocol, and optionally the version of the server software.
  The version response is formatted as `version mcsci=<protocol-version number> [server=<server-version double-quoted string>]?`, where the `server` version is optional.

- `extensions`
  Lists all available extensions.
  The server should respond with the `ack` acknowledgement, then a `extensions` response containing all available extensions.
  The extensions response is formatted as `extensions <extension count as a number> [<extension-name as a double-quoted string> <extension-version as a double-quoted string> <extension-description as a double-quoted string>]*`. The order of the extensions must be consistent during the duration of the connection, and must be the same every time the extensions command is issued. The "id" of an extension is its position in the list.

- `list-types <extension id as a number>`  
  Queries the list of type aliases that an extension provides. The server should respond with the `ack` acknowledgement then a `type-list <entension id as a number> [(<name> = <declaration>)]*`.
  For example, an extension could define types as such: `type-list 1234 (block_pos = tuple(i32, i32, i32)) (chunk_pos = tuple(i32, i32))`,

- `list-problems <extension id as a number>`  
  Queries the list of problems that an extension can solve. The server should respond with the `ack` acknowledgement then a `problem-list <entension id as a number> <problems array as a typed value>`. For the types of the problems array, see the [Typed Values](#typed-values) section.

- `setup-problem <extension id as a number> <problem name double-quoted string> [<arg name> = <arg value as a typed value>]*`  
  Sets up the server to handle a computation problem with the given name and arguments.
  The server should respond with the `ack` acknowledgement after having processed the command. It should then send a `setup-ok` or `setup-error <error as a typed value>` response.

Responses:

- `ack`  
  Sent by the server to acknowledge the receipt of a command.

- `setup-ok`  
  Sent by the server to the client when the `setup-problem` command is successfully processed.
  
- `setup-error <error as a typed value>`  
  Sent by the server to the client when the `setup-problem` command is rejected.

- `parsefail`
  Sent by the server to the client when it couldn't parse the command.

- `version mcsci=<protocol-version number> [server=<server-version double-quoted string>]?`  
  Sent by the server to the client when the `version` command is successfully processed.

- `unexpected <error as a typed value>?`  
  Sent by the server to the client when it didn't expect the command. The error message might be of any type and is optional.

- `no-such-extension <extension id as a number>`  
  Sent by the server to the client when it couldn't find an extension with the given id.

## Extensions

The protocol supports server extensions. A client can query the extensions that the server supports using the `extensions` command. In this section, when referring to the "id" of an extension, we mean its position in the list of extensions returned by the `extensions` command.
The presence of an extension on the server must not prevent clients that do not support the extension from connecting to the server or using features provided by other extensions supported by both parties. It is for this reason that extension specific communication must be done through special commands.
To send a command to an extension, the client must send the `use-extension <extension id as a number> <usage id as a number> <command>` command. The server uses the "usage id" parameter sent by the client when the extension sends responses to that invokation. The server responds with the `ack` acknowledgement after having received the use request, an `unexpected <error as a typed value>` or a `parsefail` response. Any responses from the extension are sent to the client using the `extension-response <usage id as a number> <response>` response.
This document does not provide anything about how extensions are implemented, how their commands are structured, how they are parsed, or how they are executed. That is left to the implementer of the extension. This specification does not impose any restrictions on the format of commands, responses, or the way they are parsed. This specification does not impose any restrictions on the format of the data sent between the client and the server.
While the client is expected to not send any core command before it has received an aknowledgement, or an error from the server, this does not apply to extensions, and the behaviour is left to the implementer of the extension.

## Typed Values

The protocol supports the transfer of typed data, serialized in plaintext form.
The client can get a list of available types using the `types` command, for more information see the [Core Commands / Responses](#core-commands--responses) section.

### Version 0

This version introduces the following native types:

- `string`: A string of UTF-8 characters, represented as a double-quoted string.  
  For example, `"hello world"`.
  There is no restriction on the length of the string, or how it is encoded, apart from these:
  - The string must be valid UTF-8.
  - If any backslash is encountered, it is considered to be an escape character. Standard escape sequences are supported: `\n` for a newline, `\r` for a carriage return, `\t` for a tab, `\\` for a backslash (simply putting a single backslash `\` is invalid because it is used as an escape character), `\"` for a double-quote inside the string, `\'` for a single quote, and `\u{hex}` where "hex" is the hexadecimal unicode codepoint to be inserted, for example `\u{1f3c4}` for the emoji "person surfing". The only characters that *need* to be escaped are `\"`, `\\`, `\n`, `\r`, `\t` and non-printable ASCII characters. Any other character is valid inside the string if used unescaped, although it is not recommended, and still needs to be parsed correctly if escaped.

- `i8`, `i16`, `i32`, `i64`: Respectively, 8 bits, 16 bits, 32 bits, and 64 bits signed integers. They are represented either in decimal, (e.g. `-41`), in hexadecimal (e.g. `-0x2a`), in octal (e.g. `-0o52`), in binary (e.g. `-0b1010101`), or in a custom base in the format `<type>(<value as a string>[, <radix as a base 10 integer between 2 and 36 inclusive>]?)` (e.g. `i32("-1a", 11)` for base 11, or `i64("1568")` where you can omit the radix when it is 10), depending on the value and datatype. If the datatype is not specified, then the value has the type that is the smallest one that can represent it fully. This means in order: i8, u8, i16, u16, i32, u32, i64, u64. If the server expects an datatype larger than the one provided, it may convert the given value to the expected type, provided that no information is lost.

- `u8`, `u16`, `u32`, `u64`: Respectively, 8 bits, 16 bits, 32 bits, and 64 bits unsigned integers. See the signed integer types for more information.

- `f32`, `f64`: Respectively, 32 bits and 64 bits floating point numbers. They are represented either in the decimal format (e.g. `1.0`, `3.14159`, `1e10`, `2.7e-17`, `-1.5e7`, `NaN`, `Infinity`, `-Infinity`), or using their binary representation as specified in IEEE 754 (e.g. pi is `f64(0x400921fb54442d18)`). The f32 and f64 constructors take either a string to convert to their respective datatype, or a hexadecimal u32 (respectively u64) of the binary representation of the float, as described in IEEE 754. If the datatype is not specified, it defaults to f32.

- `bool`: A boolean value, represented as `true` or `false`.

This version also introduces the following ways to construct composite types:

- `tuple([<type> [, <type>]*]?)`  
  A tuple of the given types.
  A tuple is constructed by enclosing the values in parentheses, optionally preceded by the name alias of the type if it can't be infered from the context (`tuple_type_name::<tuple value>`).
  For example, as shown later, the types `block_pos` and `chunk_pos` are aliases for `tuple(i32, i32, i32)` and `tuple(i32, i32)`, so you can construct a block position using `(<x>, <y>, <z>)` or `block_pos::(<x>, <y>, <z>)`, and a chunk position using `(<x>, <z>)` or `chunk_pos::(<x>, <z>)`.

- `list(<type>)`
  A list of elements of the given type.
  A list is constructed by enclosing the values in square brackets, optionally preceded by the name alias of the type if it can't be infered from the context (`list_type_name::<list value>`).

- `array(<type>, <length as u32>)`
  An array of the given type with the given length.
  An array is constructed by enclosing the values in square brackets, optionally receded by the name alias of the type if it can't be infered from the context (`array_type_name::<array value>`).

- `enum(<constructor name> [(<type>)]? [, <constructor name> [(<type>)]?]*)`  
  An enumeration type.
  An instance of an enumeration type is constructed by using one of its constructors, optionally preceded by the name alias of the type if it can't be infered from the context (`enum_type_name::constructor[(<value>)]?`). The constructor can optionally take a value of the given type.
  For example, as shown later, the type `end_pillar_height_hint` is an alias for `enum(Unknown, Big, Medium, Small, MediumSmall, MediumBig, Exact(i32), Range((i32, i32)))`, so you can construct an end pillar height hint using `Unknown`, `Big`, `Medium`, `Small`, `MediumSmall`, `MediumBig`, `Exact(<height>)`, or `Range((<min>, <max>))`.

As such, some composite types that are available on v0 servers:

- `extension_info`: `tuple(string, string, string, list(string), list(string))`, in order: name, version, description, authors, extension-specific commands
- `problem_arg`: `tuple(string, bool, string)`, in order: argument name, is it optional ?, a type alias (note that extensions define their own type aliases)
- `problem_description` : `tuple(string, string, list(problem_arg))`, in order: name, description, arguments
- `extension_problem_list`: `list(problem_description)`