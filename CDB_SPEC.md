# Commando Database (CDB) Specification

# Version 0

Note: version 0 is *the* unstable version.

## Types

Values starting with `0x` are to be read as a hexadecimal number.

| Type        | Description                                                                                                                  |
|-------------|------------------------------------------------------------------------------------------------------------------------------|
| Magic       | The magic number, a sequence of bytes to recognize the format. Currently corresponds to: \[`0x7F`, `0x43`, `0x44`, `0x42`\]. |
| Byte        | An unsigned 8 bit integer.                                                                                                   |
| UInt        | An unsigned, little endian, 32 bit integer.                                                                                  |
| Newline     | A new line `Byte` with value of `0xA`.                                                                                       |
| ETX         | An end of text `Byte` with value of `0x3`.                                                                                   |
| String(len) | A sequence of bytes of maximum length defined by `len`, representing a UTF-8 string.                                         |

## Format

The database format itself.

| Name           | Type                     | Description                                                                                                                                     | Repeatable             |
|----------------|--------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------|------------------------|
| magic_number   | Magic                    | The magic number to identify the format.                                                                                                        | False                  |
| version_number | UInt                     | The version of the database format.                                                                                                             | False                  |
| command_length | Byte                     | The length of the command_name String.                                                                                                          | `n` times              |
| command_name   | String(command_length+1) | The name of the command provided by package_name(s). Length is greater than zero.                                                               | `n` times              |
| package_length | UInt                     | The sum of the length of every package_name with an added newline. Equivalent to $\sum^{m} (len + 1)$ with `len` being package_name(s') length. | `n` times              |
| package_name   | String(package_length+1) | The name of a package that provides command_name. Length is greater than zero.                                                                  | `m` times for each `n` |
| end_package    | Newline                  | The newline that separates the package_name(s).                                                                                                 | `m` times for each `n` |
| end_command    | ETX                      | An `ETX` character signaling the end of this command_name's section.                                                                            | `n` times              |

Repeatable fields always follow the same order.

The structure of the database represents a *1:N* relationship between a
single command and many packages.
