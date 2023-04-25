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
function ExeDat:read_file(path: string): string
```

Reads the contents of a file out of the .dat file.

Previous calls to `write_file` are visible to subsequent calls to `read_file`.

### `ExeDat:write_file`

```lua
function ExeDat:write_file(path: string, contents: string): string
```

Writes the file data into the .dat file.

Note that this does not mutate the original .dat file on disk, but for all intents and purposes to both the game and the mod loader it does.

## `chaudloader.mpak`

### `chaudloader.exedat.unpack`

```lua
function chaudloader.mpak.unpack(map_contents: string, mpak_contents: string): Mpak
```

Unmarshals an .map + .mpak file.

### `Mpak:__index`

```lua
Mpak[rom_addr: integer] = string
```

Inserts an entry at the given ROM address into the mpak.

Existing entries will be clobbered. If contents is nil, the entry will be deleted.

### `Mpak:__newindex`

```lua
Mpak[rom_addr: integer]: string
```

Reads an entry at the given ROM address.

### `Mpak:__pairs`

```lua
pairs(Mpak): function (): integer, string
```

Iterates through all entries of an mpak.

### `Mpak:pack`

```lua
function Mpak:pack(): string, string
```

Marshals an mpak back into .map + .mpak format.

## `chaudloader.bytearray`

### `chaudloader.bytearray.unpack`

```lua
function chaudloader.bytearray.unpack(raw: string): ByteArray
```

Copies a string into a byte array.

Unlike Lua tables and strings, byte arrays are 0-indexed: this is such that offsets in the byte array will match up directly to file offsets for convenience.

### `chaudloader.bytearray.filled`

```lua
function chaudloader.bytearray.filled(v: integer, n: integer): ByteArray
```

Creates a new byte array filled with `n` bytes of `v`.

### `ByteArray:__concat`

```lua
ByteArray(...) .. ByteArray(...): ByteArray
```

Concatenates two byte arrays together and returns the concatenated byte array.

### `ByteArray:__eq`

```lua
ByteArray(...) == ByteArray(...): bool
```

Compares two byte arrays for byte-for-byte equality.

### `ByteArray:clone`

```lua
ByteArray:clone(): ByteArray
```

Clones the byte array into a new, unshared byte array.

### `ByteArray:len`

```lua
ByteArray:len(): integer
```

Returns the length of the byte array, in bytes.

### `ByteArray:pack`

```lua
ByteArray:pack(): string
```

Packs the byte array back into a Lua string.

### `ByteArray:get_string`

```lua
ByteArray:get_string(i: integer, n: integer): string
```

Gets the bytes at [`i`, `n`) as a string.

If `i + n` is greater than the length of the byte array, an error will be raised.

### `ByteArray:set_string`

```lua
ByteArray:set_string(i: integer, s: string)
```

Sets the bytes starting at `i` to the bytes in `s`.

If `i + #s` is greater than the length of the byte array, an error will be raised.

### `ByteArray:get_bytearray`

```lua
ByteArray:get_bytearray(i: integer, n: integer): ByteArray
```

Gets the bytes at [`i`, `n`) as a byte array.

If `i + n` is greater than the length of the byte array, an error will be raised.

### `ByteArray:set_bytearray`

```lua
ByteArray:set_bytearray(i: integer, ba: ByteArray)
```

Sets the bytes starting at `i` to the bytes in `ba`.

If `i + ba:len()` is greater than the length of the byte array, an error will be raised.

### `ByteArray:get_{u8,u16_le,u32_le,i8,i16_le,i32_le}`

```lua
ByteArray:get_u8(i: integer): integer
ByteArray:get_u16_le(i: integer): integer
ByteArray:get_u32_le(i: integer): integer
ByteArray:get_i8(i: integer): integer
ByteArray:get_i16_le(i: integer): integer
ByteArray:get_i32_le(i: integer): integer
```

Gets the bytes at `i` as the given integer type.

If `i + width` is greater than the length of the byte array, an error will be raised.

### `ByteArray:set_{u8,u16_le,u32_le,i8,i16_le,i32_le}`

```lua
ByteArray:set_u8(i: integer, v: integer)
ByteArray:set_u16_le(i: integer, v: integer)
ByteArray:set_u32_le(i: integer, v: integer)
ByteArray:set_i8(i: integer, v: integer)
ByteArray:set_i16_le(i: integer, v: integer)
ByteArray:set_i32_le(i: integer, v: integer)
```

Sets the bytes starting at `i` to the integer `v`.

If `i + width` is greater than the length of the byte array, an error will be raised.

## `chaudloader.msg`

### `chaudloader.msg.unpack`

```lua
function chaudloader.msg.unpack(raw: string): {[integer]: string}
```

Unmarshals msg data.

### `chaudloader.msg.pack`

```lua
function chaudloader.msg.pack(entries: {[integer]: string}): string
```

Marshals msg data.

## `chaudloader.modfiles`

Functions for accessing files from the mod's directory.

### `chaudloader.modfiles.read_file`

```lua
function chaudloader.modfiles.read_file(path: string): string
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
function chaudloader.unsafe.write_process_memory(addr: integer, buf: string)
```

Writes directly into process memory.

### `chaudloader.unsafe.read_process_memory`

```lua
function chaudloader.unsafe.read_process_memory(addr: integer, n: integer): string
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

See [chaudloader/src/mods/lua/lib/compat.lua](chaudloader/src/mods/lua/lib/compat.lua) for compatibility functions with old versions of chaudloader. Note that these functions may be removed at any time.

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