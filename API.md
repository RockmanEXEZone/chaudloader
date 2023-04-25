# API guide

## Global functions

### `require`

```lua
function require(name: string): any
```

Requires a module from the mods directory.

If `unsafe = true` is set in `info.toml`, `require` also may load Lua DLLs of the form `<name>.dll` from the mods directory.

If the name contains dots (`.`), they will be translated to slashes for paths (`/`). If the name is for a Lua DLL, they will be replaced with underscores (`_`) in the loader function. For example, for a library named `foo.bar`:

-   **Path:** `foo/bar.lua` (or `foo/bar.dll`)
-   **DLL entry point:** `luaopen_foo_bar`

For more information on writing Lua libraries, see https://www.lua.org/pil/26.2.html. If you don't particularly feel like using any Lua features, you may define your luaopen function like so:

```c
int luaopen_mylibrary(void* unused) {
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

Game name ("Vol1" or "Vol2").

### `chaudloader.GAME_ENV.exe_sha256`

```lua
chaudloader.GAME_ENV.exe_sha256: string
```

SHA256 of the EXE.

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

## `chaudloader.ExeDat`

```lua
function chaudloader.ExeDat(dat_filename: string): ExeDat
```

Opens an exe/data .dat file located in exe/data (e.g. `exe6.dat`).

### `chaudloader.ExeDat:read_file`

```lua
function chaudloader.ExeDat:read_file(path: string): string
```

Reads the contents of a file out of the .dat file.

Previous calls to write_exe_dat_contents are visible to subsequent calls to read_exe_dat_contents.

### `chaudloader.ExeDat:write_file`

```lua
function chaudloader.ExeDat:write_file(path: string, contents: string): string
```

Writes the file data into the .dat file.

Note that this does not mutate the original .dat file on disk, but for all intents and purposes to both the game and the mod loader it does.

## `chaudloader.Mpak`

```lua
function chaudloader.Mpak(map_contents: string, mpak_contents: string): Mpak
```

Unmarshals an .map + .mpak file.

### `chaudloader.Mpak:__index`

```lua
chaudloader.Mpak[rom_addr: integer] = string
```

Inserts an entry at the given ROM address into the mpak. Existing entries will be clobbered. If contents is nil, the entry will be deleted.

### `chaudloader.Mpak:__newindex`

```lua
chaudloader.Mpak[rom_addr: integer]: string
```

Reads an entry at the given ROM address.

### `chaudloader.Mpak:__pairs`

```lua
pairs(chaudloader.Mpak): (integer, string)
```

Iterates through all entries of an mpak.

### `chaudloader.Mpak:pack`

```lua
function chaudloader.Mpak:pack(): (string, string)
```

Marshals an mpak back into .map + .mpak format.

## `chaudloader.ByteArray`

```lua
function chaudloader.ByteArray(raw: string): ByteArray
```

Copies a string into a byte array. Note that, unlike Lua tables and strings, byte arrays are 0-indexed: this is such that offsets in the byte array will match up directly to file offsets for convenience.

### `chaudloader.ByteArray:__concat`

```lua
chaudloader.ByteArray(...) .. chaudloader.ByteArray(...): ByteArray
```

Concatenates two byte arrays together and returns the concatenated byte array.

### `chaudloader.ByteArray:__eq`

```lua
chaudloader.ByteArray(...) == chaudloader.ByteArray(...): bool
```

Compares two byte arrays for byte-for-byte equality.

### `chaudloader.ByteArray:len`

```lua
chaudloader.ByteArray:len(): integer
```

Returns the length of the byte array, in bytes.

### `chaudloader.ByteArray:pack`

```lua
chaudloader.ByteArray:pack(): string
```

Packs the byte array back into a Lua string.

### `chaudloader.ByteArray:get_string`

```lua
chaudloader.ByteArray:get_string(i: integer, n: integer): string
```

Gets the bytes at [`i`, `n`) as a string. If `i + n` is greater than the length of the byte array, an error will be raised.

### `chaudloader.ByteArray:set_string`

```lua
chaudloader.ByteArray:set_string(i: integer, s: string)
```

Sets the bytes starting at `i` to the bytes in `s`. If `i + #s` is greater than the length of the byte array, an error will be raised.

### `chaudloader.ByteArray:get_bytearray`

```lua
chaudloader.ByteArray:get_bytearray(i: integer, n: integer): ByteArray
```

Gets the bytes at [`i`, `n`) as a byte array. If `i + n` is greater than the length of the byte array, an error will be raised.

### `chaudloader.ByteArray:set_bytearray`

```lua
chaudloader.ByteArray:set_bytearray(i: integer, ba: ByteArray)
```

Sets the bytes starting at `i` to the bytes in `ba`. If `i + ba:len()` is greater than the length of the byte array, an error will be raised.

### `chaudloader.ByteArray:get_{u8,u16_le,u32_le,i8,i16_le,i32_le}`

```lua
chaudloader.ByteArray:get_u8(i: integer): integer
chaudloader.ByteArray:get_u16_le(i: integer): integer
chaudloader.ByteArray:get_u32_le(i: integer): integer
chaudloader.ByteArray:get_i8(i: integer): integer
chaudloader.ByteArray:get_i16_le(i: integer): integer
chaudloader.ByteArray:get_i32_le(i: integer): integer
```

Gets the bytes at `i` as the given integer type. If `i + width` is greater than the length of the byte array, an error will be raised.

### `chaudloader.ByteArray:set_{u8,u16_le,u32_le,i8,i16_le,i32_le}`

```lua
chaudloader.ByteArray:set_u8(i: integer, v: integer)
chaudloader.ByteArray:set_u16_le(i: integer, v: integer)
chaudloader.ByteArray:set_u32_le(i: integer, v: integer)
chaudloader.ByteArray:set_i8(i: integer, v: integer)
chaudloader.ByteArray:set_i16_le(i: integer, v: integer)
chaudloader.ByteArray:set_i32_le(i: integer, v: integer)
```

Sets the bytes starting at `i` to the integer `v`. If `i + width` is greater than the length of the byte array, an error will be raised.

## msg data functions

### `chaudloader.unpack_msg`

```lua
function chaudloader.unpack_msg(raw: string): {[integer]: string}
```

Unmarshals msg data.

### `chaudloader.pack_msg`

```lua
function chaudloader.pack_msg(entries: {[integer]: string}): string
```

Marshals msg data.

## Mod file functions

### `chaudloader.read_mod_file`

```lua
function chaudloader.read_mod_file(path: string): string
```

Reads the contents of a file from the mod folder.

### `chaudloader.list_mod_directory`

```lua
function chaudloader.list_mod_directory(path: string): {[integer]: string}
```

Lists the contents of a directory from the mod folder.

### `chaudloader.get_mod_file_metadata`

```lua
function chaudloader.get_mod_file_metadata(path: string): {type: "dir" | "file", size: integer}
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

<details>
<summary>Deprecated API</summary>

## Deprecated API

These APIs have been deprecated and may be removed in a future version. You should use the appropriate alternative.

### `chaudloader.unsafe.init_mod_dll`

```lua
function chaudloader.unsafe.init_mod_dll(path: string, userdata: string)
```

**Deprecated:** See `require`.

Loads a library from the mod folder and call its chaudloader_init function.

```rust
chaudloader_init: unsafe extern "system" fn(userdata: *const u8, n: usize) -> bool
```

### `bnlc_mod_loader.read_exe_dat_contents`

```lua
function bnlc_mod_loader.read_exe_dat_contents(dat_filename: string, path: string): string
```

**Deprecated:** See `chaudloader.ExeDat:read_file`.

Reads the contents of a file out of a .dat file located in exe/data (e.g. `exe6.dat`).

Previous calls to write_exe_dat_contents are visible to subsequent calls to read_exe_dat_contents.

### `bnlc_mod_loader.write_exe_dat_contents`

```lua
function bnlc_mod_loader.write_exe_dat_contents(dat_filename: string, path: string, contents: string)
```

**Deprecated:** See `chaudloader.ExeDat:write_file`.

Writes the given data into a zip .dat file located in exe/data.

Note that this does not mutate the original .dat file on disk, but for all intents and purposes to both the game and the mod loader it does.

### `bnlc_mod_loader.read_mod_contents`

**Deprecated:** See `chaudloader.read_mod_file`.

```lua
function bnlc_mod_loader.read_mod_contents(path: string): string
```

Reads the contents of a file from the mod folder.

</details>

## Convenience functions

```lua
chaudloader.util: {
    -- whatever is in chaudloader/src/mods/lua/chaudloader/util.lua
}
```
