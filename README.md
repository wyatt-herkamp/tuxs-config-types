# Config Types

Types to help you build a user-friendly configuration system.

## Notes

All types use [thiserror](https://github.com/dtolnay/thiserror) for Error types. Uses [strum](https://github.com/Peternator7/strum) to help with enums

When Regex is used [once_cell](https://github.com/matklad/once_cell) is also required to hold the Regex

All modules are isolated. Meaning you can copy the code within to use for your project without having to depend on this entire project

| Path                                                                                                             | Helps with                                       | Required Features               |
| ---------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ | ------------------------------- |
| [chrono_types::duration](https://github.com/wyatt-herkamp/config_types/blob/master/src/chrono_types/duration.rs) | Building Duration with different suffixes        | Chrono, Regex, Once_Cell, strum |
| [size_config](https://github.com/wyatt-herkamp/config_types/blob/master/src/size_config.rs)                      | Building a Size String such as 100mb, 100b, 10gb | Regex, Once_cell, strum         |
