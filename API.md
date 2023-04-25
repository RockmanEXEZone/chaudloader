# API guide

## Global functions

### `require`

```lua
function require(name: string): any
```

Requires a module from the mods directory.

If `unsafe = true` is set in `info.toml`, `require` also may load Lua DLLs of the form `{name}.dll` from the mods directory.

If the name contains dots (`.`), they will be translated to slashes for paths (`/`). If the name is for a Lua DLL, they will be replaced with underscores (`_`) in the loader function. For example, for a library named `foo.bar`:

-   **Path:** `foo/bar.lua` (or `foo/bar.dll`)
-   **DLL entry point:** `luaopen_foo_bar`

For more information on writing Lua libraries, see https://www.lua.org/pil/26.2.html. If you don't particularly feel like using any Lua features, you may define your luaopen function like so:

```c
__declspec(dllexport) int luaopen_mylibrary(void* unused) {
    // Do all your logic here.
    return 0;
}
```

### `print`

```lua
function print(...)
```

Prints a log line.

## Execution environment

### `chaudloader.GAME_ENV.name`

```lua
chaudloader.GAME_ENV.name: string
```

Game name (`"Vol1"` or `"Vol2"`).

### `chaudloader.GAME_ENV.exe_sha256`

```lua
chaudloader.GAME_ENV.exe_sha256: string
```

SHA256 of the EXE.

This may be useful to ensure your mod is loaded for the correct version of the binary if you are hooking hard-coded addresses in the binary.

### `chaudloader.MOD_ENV.name`

```lua
chaudloader.MOD_ENV.name: string
```

Name of the current mod.

### `chaudloader.MOD_ENV.path`

```lua
chaudloader.MOD_ENV.path: string
```

Path to the current mod.

## `chaudloader.exedat`

### `chaudloader.exedat.open`

```lua
function chaudloader.exedat.open(dat_filename: string): ExeDat
```

Opens an exe/data .dat file located in exe/data (e.g. `exe6.dat`).

### `ExeDat:read_file`

```lua
function ExeDat:read_file(path: string): Buffer
```

Reads the contents of a file out of the .dat file.

Previous calls to `write_file` are visible to subsequent calls to `read_file`.

### `ExeDat:write_file`

```lua
function ExeDat:write_file(path: string, contents: Buffer)
```

Writes the file data into the .dat file.

Note that this does not mutate the original .dat file on disk, but for all intents and purposes to both the game and the mod loader it does.

## `chaudloader.mpak`

### `chaudloader.exedat.unpack`

```lua
function chaudloader.mpak.unpack(map_contents: Buffer, mpak_contents: Buffer): Mpak
```

Unmarshals an .map + .mpak file.

### `Mpak:__index`

```lua
Mpak[rom_addr: integer] = Buffer
```

Inserts an entry at the given ROM address into the mpak.

Existing entries will be clobbered. If contents is nil, the entry will be deleted.

### `Mpak:__newindex`

```lua
Mpak[rom_addr: integer]: Buffer
```

Reads an entry at the given ROM address.

### `Mpak:__pairs`

```lua
pairs(Mpak): function (): integer, Buffer
```

Iterates through all entries of an mpak.

### `Mpak:pack`

```lua
function Mpak:pack(): Buffer, Buffer
```

Marshals an mpak back into .map + .mpak format.

## `chaudloader.buffer`

Buffers are mutable arrays of bytes with immutable length.

Unlike Lua tables and strings, buffers are 0-indexed: this is such that offsets in the buffer will match up directly to file offsets for convenience.

Note that a bunch of standard Lua metamethods that expose indexing (i.e. `__ipairs`, `__len`, `__index`, `__newindex`) are not implemented to avoid confusion with 1-based Lua tables.

### `chaudloader.buffer.from_string`

```lua
function chaudloader.buffer.from_string(raw: string): Buffer
```

Copies a string into a buffer.

### `chaudloader.buffer.filled`

```lua
function chaudloader.buffer.filled(v: integer, n: integer): Buffer
```

Creates a new buffer filled with `n` bytes of `v`.

### `Buffer:__concat`

```lua
Buffer(...) .. Buffer(...): Buffer
```

Concatenates two buffers together and returns the concatenated buffer.

### `Buffer:__eq`

```lua
Buffer(...) == Buffer(...): bool
```

Compares two buffers for byte-for-byte equality.

### `Buffer:clone`

```lua
Buffer:clone(): Buffer
```

Clones the buffer into a new, unshared buffer.

### `Buffer:len`

```lua
Buffer:len(): integer
```

Returns the length of the buffer, in bytes.

### `Buffer:to_string`

```lua
Buffer:to_string(): string
```

Packs the buffer back into a Lua string.

### `Buffer:get_string`

```lua
Buffer:get_string(i: integer, n: integer): string
```

Gets the bytes at [`i`, `n`) as a string.

If `i + n` is greater than the length of the buffer, an error will be raised.

### `Buffer:set_string`

```lua
Buffer:set_string(i: integer, s: string)
```

Sets the bytes starting at `i` to the bytes in `s`.

If `i + #s` is greater than the length of the buffer, an error will be raised.

### `Buffer:get_buffer`

```lua
Buffer:get_buffer(i: integer, n: integer): Buffer
```

Gets the bytes at [`i`, `n`) as a buffer.

If `i + n` is greater than the length of the buffer, an error will be raised.

### `Buffer:set_buffer`

```lua
Buffer:set_buffer(i: integer, buf: Buffer)
```

Sets the bytes starting at `i` to the bytes in `buf`.

If `i + buf:len()` is greater than the length of the buffer, an error will be raised.

### `Buffer:get_{u8,u16_le,u32_le,i8,i16_le,i32_le}`

```lua
Buffer:get_u8(i: integer): integer
Buffer:get_u16_le(i: integer): integer
Buffer:get_u32_le(i: integer): integer
Buffer:get_i8(i: integer): integer
Buffer:get_i16_le(i: integer): integer
Buffer:get_i32_le(i: integer): integer
```

Gets the bytes at `i` as the given integer type.

If `i + width` is greater than the length of the buffer, an error will be raised.

### `Buffer:set_{u8,u16_le,u32_le,i8,i16_le,i32_le}`

```lua
Buffer:set_u8(i: integer, v: integer)
Buffer:set_u16_le(i: integer, v: integer)
Buffer:set_u32_le(i: integer, v: integer)
Buffer:set_i8(i: integer, v: integer)
Buffer:set_i16_le(i: integer, v: integer)
Buffer:set_i32_le(i: integer, v: integer)
```

Sets the bytes starting at `i` to the integer `v`.

If `i + width` is greater than the length of the buffer, an error will be raised.

## `chaudloader.msg`

### `chaudloader.msg.unpack`

```lua
function chaudloader.msg.unpack(raw: Buffer): {[integer]: Buffer}
```

Unmarshals msg data.

### `chaudloader.msg.pack`

```lua
function chaudloader.msg.pack(entries: {[integer]: Buffer}): Buffer
```

Marshals msg data.

## `chaudloader.modfiles`

Functions for accessing files from the mod's directory.

### `chaudloader.modfiles.read_file`

```lua
function chaudloader.modfiles.read_file(path: string): Buffer
```

Reads the contents of a file from the mod folder.

### `chaudloader.modfiles.list_directory`

```lua
function chaudloader.modfiles.list_directory(path: string): {[integer]: string}
```

Lists the contents of a directory from the mod folder.

### `chaudloader.modfiles.get_file_metadata`

```lua
function chaudloader.modfiles.get_file_metadata(path: string): {type: "dir" | "file", size: integer}
```

Gets the metadata of a file from the mod folder.

## `chaudloader.unsafe`

Your mod must have `unsafe = true` in `info.toml` to use these functions.

### `chaudloader.unsafe.write_process_memory`

```lua
function chaudloader.unsafe.write_process_memory(addr: integer, buf: Buffer)
```

Writes directly into process memory.

### `chaudloader.unsafe.read_process_memory`

```lua
function chaudloader.unsafe.read_process_memory(addr: integer, n: integer): Buffer
```

Reads directly from process memory.

## Convenience functions

```lua
chaudloader.util: {
    -- whatever is in chaudloader/src/mods/lua/lib/chaudloader/util.lua
}
```

See [chaudloader/src/mods/lua/lib/chaudloader/util.lua](chaudloader/src/mods/lua/lib/chaudloader/util.lua) for functions available in this namespace.

## Deprecated API

### Compatibility shims

See [chaudloader/src/mods/lua/compat.lua](chaudloader/src/mods/lua/compat.lua) for compatibility functions with old versions of chaudloader. Note that these functions may be removed at any time.

### Other

#### `chaudloader.unsafe.init_mod_dll`

```lua
function chaudloader.unsafe.init_mod_dll(path: string, userdata: string)
```

**Deprecated:** See `require`.

Loads a library from the mod folder and call its chaudloader_init function.

The function should have the following signature:

```c
__declspec(dllexport) bool chaudloader_init(const char* userdata, n: size_t)
```
