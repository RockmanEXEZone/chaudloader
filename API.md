# API guide

## Global functions

### `require`

```lua
function require(name: string): any
```

Requires a package from the mod directory.

If `unsafe = true` is set in `info.toml`, `require` also may load Lua DLLs of the form `{name}.dll` from the mods directory. If the name contains dots (`.`), they will be replaced with underscores (`_`) to resolve the loader function, named `luaopen_<package name>`.

The search order is as follows:

-   **Exact path:** `require("foo.lua")` will require a file named `foo.lua`, and `require("foo.dll")` will require a file named `foo.dll` exposing `luaopen_foo`.

-   **Short path:**

    -   `require("foo.lua")` will try to require a file named `foo.lua.lua`, then try to require a file named `foo.lua.dll` exposing `luaopen_foo_lua`.
    -   `require("foo/bar")` will try to require a file named `foo/bar.lua`, then try to require a file named `foo/bar.dll` exposing `luaopen_bar`.

-   **Dotted name:**

    -   `require("foo.lua")` will try to require a file named `foo/lua.lua`, then try to require a file named `foo/lua.dll` exposing `luaopen_foo_lua`.

For more information on writing Lua libraries, see https://www.lua.org/pil/26.2.html. If you don't particularly feel like using any Lua features, you may define your luaopen function like so for e.g. a library named `mylibrary`:

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

### `chaudloader.GAME_ENV.exe_crc32`

```lua
chaudloader.GAME_ENV.exe_crc32: integer
```

CRC32 of the EXE.

This may be useful to ensure your mod is loaded for the correct version of the binary if you are hooking hard-coded addresses in the binary.

### `chaudloader.GAME_ENV.sections`

```lua
chaudloader.GAME_ENV.sections.text: table | nil
chaudloader.GAME_ENV.sections.text.address: integer
chaudloader.GAME_ENV.sections.text.size: integer
```

Current virtual memory address and size of the game's `.text` section. If chaudloader could not determine the location and size of the `.text` section, then `chaudloader.GAME_ENV.text` returns `nil`.

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

### `chaudloader.mpak.unpack`

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

## `chaudloader.bnk`

Functions for loading new bnk files.

### `chaudloader.bnk.load_bnk`

```lua
function chaudloader.bnk.load_bnk(path: string)
```

Loads the bnk file from `path` after Vol1Global.bnk or Vol2Global.bnk.


## `chaudloader.pck`

Functions for replacing playback of music/voices from pck files and loading new pck files.

### `chaudloader.pck.replace_wem`

```lua
function chaudloader.pck.replace_wem(id: integer, path: string, language_id: integer)
```
Replaces attempts to play the wem file with `id` in the game's original pck files with the wem from `path` for the specific `language_id`.

The `language_id`s are:
```
SFX = 0
Japanese = 1
Chinese = 2
English = 3
```

### `chaudloader.pck.replace_wem_sfx`

```lua
function chaudloader.pck.replace_wem_sfx(id: integer, path: string)
```

Replaces attempts to play the sfx wem file with `id` in the game's original pck files with the wem from `path`.

### `chaudloader.pck.replace_wem_japanese`

```lua
function chaudloader.pck.replace_wem_japanese(id: integer, path: string)
```

Replaces attempts to play the Japanese wem file with `id` in the game's original pck files with the wem from `path`.

### `chaudloader.pck.replace_wem_chinese`

```lua
function chaudloader.pck.replace_wem_chinese(id: integer, path: string)
```

Replaces attempts to play the Chinese wem file with `id` in the game's original pck files with the wem from `path`.

### `chaudloader.pck.replace_wem_english`

```lua
function chaudloader.pck.replace_wem_english(id: integer, path: string)
```

Replaces attempts to play the English wem file with `id` in the game's original pck files with the wem from `path`.

### `chaudloader.pck.load_pck`

```lua
function chaudloader.pck.load_pck(path: string)
```

Loads the pck file from `path` after Vol1 or Vol2.pck. Any wems with IDs that match the original pck play in place of the original.

## `chaudloader.buffer`

Buffers are mutable arrays of bytes with immutable length.

Unlike Lua tables and strings, buffers are 0-indexed: this is such that offsets in the buffer will match up directly to file offsets for convenience.

Note that a bunch of standard Lua metamethods that expose indexing (i.e. `__ipairs`, `__len`, `__index`, `__newindex`) are not implemented to avoid confusion with 1-based Lua tables.

### `chaudloader.buffer.from_string`

```lua
function chaudloader.buffer.from_string(raw: string): Buffer
```

Copies a string into a buffer.

### `chaudloader.buffer.from_string`

```lua
function chaudloader.buffer.from_u8_table(raw: {[integer]: integer}): Buffer
```

Copies a table of u8s into a buffer.

### `chaudloader.buffer.filled`

```lua
function chaudloader.buffer.filled(v: integer, n: integer): Buffer
```

Creates a new buffer filled with `n` bytes of `v`.

### `chaudloader.buffer.empty`

```lua
function chaudloader.buffer.empty(): Buffer
```

Creates a new buffer with zero length.

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

### `Buffer:slice`

```lua
Buffer:slice(i: integer, n: integer): Buffer
```

Gets the bytes at [`i`, `n`) as a shared view into buffer.

If `i + n` is greater than the length of the buffer, an error will be raised.

If you need a cloned view, use `Buffer:get`.

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

### `Buffer:get`

```lua
Buffer:get(i: integer, n: integer): Buffer
```

Gets the bytes at [`i`, `n`) as a cloned buffer.

If `i + n` is greater than the length of the buffer, an error will be raised.

If you need a shared view, use `Buffer:slice`.

### `Buffer:set`

```lua
Buffer:set(i: integer, buf: Buffer)
```

Sets the bytes starting at `i` to the bytes in `buf`.

If `i + buf:len()` is greater than the length of the buffer, an error will be raised.

### `Buffer:get_{u8,u16_le,u32_le,uq16_16,i8,i16_le,i32_le,iq16_16}`

```lua
Buffer:get_u8(i: integer): integer
Buffer:get_u16_le(i: integer): integer
Buffer:get_u32_le(i: integer): integer
Buffer:get_uq16_16_le(i: integer): number
Buffer:get_i8(i: integer): integer
Buffer:get_i16_le(i: integer): integer
Buffer:get_i32_le(i: integer): integer
Buffer:get_iq16_16_le(i: integer): number
```

Gets the bytes at `i` as the given type.

If `i + width` is greater than the length of the buffer, an error will be raised.

### `Buffer:set_{u8,u16_le,u32_le,uq16_16,i8,i16_le,i32_le,iq16_16}`

```lua
Buffer:set_u8(i: integer, v: integer)
Buffer:set_u16_le(i: integer, v: integer)
Buffer:set_u32_le(i: integer, v: integer)
Buffer:set_uq16_16_le(i: integer, v: number)
Buffer:set_i8(i: integer, v: integer)
Buffer:set_i16_le(i: integer, v: integer)
Buffer:set_i32_le(i: integer, v: integer)
Buffer:set_iq16_16_le(i: integer, v: number)
```

Sets the bytes starting at `i` to the value `v`.

If `i + width` is greater than the length of the buffer, an error will be raised.

### `chaudloader.buffer.new_builder`

```lua
chaudloader.buffer.new_builder(): BufferBuilder
```

Creates a mutable length mutable array for appending.

### `BufferBuilder:tell`

```lua
BufferBuilder:tell(): integer
```

Gets the current length of the builder.

### `BufferBuilder:build`

```lua
BufferBuilder:build(): Buffer
```

Finalizes the builder into a buffer.

### `BufferBuilder:write`

```lua
BufferBuilder:write(buf: Buffer)
```

Appends a buffer to this builder.

### `BufferBuilder:write_string`

```lua
BufferBuilder:write_string(s: string)
```

Appends a string to this builder.

### `BufferBuilder:write_{u8,u16_le,u32_le,uq16_16,i8,i16_le,i32_le,iq16_16}`

```lua
BufferBuilder:write_u8(v: integer)
BufferBuilder:write_u16_le(v: integer)
BufferBuilder:write_u32_le(v: integer)
BufferBuilder:write_uq16_16_le(v: number)
BufferBuilder:write_i8(v: integer)
BufferBuilder:write_i16_le(v: integer)
BufferBuilder:write_i32_le(v: integer)
BufferBuilder:write_iq16_16_le(v: number)
```

Appends a number to the buffer.

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

### `chaudloader.unsafe.alloc_executable_memory`

```lua
function chaudloader.unsafe.alloc_executable_memory(buf: Buffer): integer
```

Allocates and copies a Buffer into a W^X memory page.

### `chaudloader.unsafe.free_executable_memory`

```lua
function chaudloader.unsafe.free_executable_memory(addr: integer)
```

Frees memory allocated by `alloc_executable_memory`.

## Convenience functions

```lua
chaudloader.util: {
    -- whatever is in chaudloader/src/mods/lua/lib/chaudloader/util.lua
}
```

See [chaudloader/src/mods/lua/lib/chaudloader/util.lua](chaudloader/src/mods/lua/lib/chaudloader/util.lua) for functions available in this namespace.

## DLL mod functions

### `on_game_load`

This function is called when a Battle Network game is loaded. It is called after the ROM is loaded and memory is initialized, but before the game has been run.

The function should have the following signature:

```c
__declspec(dllexport) void on_game_load(int game, GBAState* gba_state) {
    // Do all your logic here.
}
```
`game`: This is the BN game being loaded. It can contain the values:
* Battle Network 1 = 0
* Battle Network 2 = 2
* Battle Network 3 White = 3
* Battle Network 3 Blue = 4
* Battle Network 4 Red Sun = 5
* Battle Network 4 Blue Moon = 6
* Battle Network 5 Team ProtoMan = 7
* Battle Network 5 Team Colonel = 8
* Battle Network 6 Cybeast Gregar = 9
* Battle Network 6 Cybeast Falzar = 10

A basic enum for this is:

```c
enum class MMBNGame : int {
    BN1 = 0,
    Unused,
    BN2,
    BN3_White,
    BN3_Blue,
    BN4_RedSun,
    BN4_BlueMoon,
    BN5_ProtoMan,
    BN5_Colonel,
    BN6_Gregar,
    BN6_Falzar,
};
```
`gba_state`: Pointer to the GBA state struct. A basic definition is:
```c
struct GBAState {
    uint32_t r0;
    uint32_t r1;
    uint32_t r2;
    uint32_t r3;
    uint32_t r4;
    uint32_t r5;
    uint32_t r6;
    uint32_t r7;
    uint32_t r8;
    uint32_t r9;
    uint32_t r10;
    uint32_t r11;
    uint32_t r12;
    uint32_t r13;
    uint32_t r14;
    uint32_t r15;
    uint32_t cpuFlags;
    uint32_t flagsImplicitUpdate;
    uint8_t* memory;
}
```


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
