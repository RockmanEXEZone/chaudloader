local exports = {}

-- Unpacks an .map and .mpak for loading, calls a function on it, then writes it back when complete.
function exports.edit_mpak(dat, name, cb)
    local mpak = chaudloader.mpak.unpack(
        dat:read_file(name .. ".map"),
        dat:read_file(name .. ".mpak")
    )
    cb(mpak)
    local raw_map, raw_mpak = mpak:pack()
    dat:write_file(name .. ".map", raw_map)
    dat:write_file(name .. ".mpak", raw_mpak)
end

-- Reads a file as a ByteArray and saves it back when done.
function exports.edit_as_bytearray(dat, path, cb)
    local ba = chaudloader.bytearray.unpack(dat:read_file(path))
    cb(ba)
    dat:write_file(path, ba:pack())
end

-- Unpacks msg data, calls a function on it, then writes it back when complete.
function exports.edit_msg(mpak, address, cb)
    local msg = chaudloader.msg.unpack(mpak[address])
    cb(msg)
    mpak[address] = chaudloader.msg.pack(msg)
end

-- Merges two messages together, preferring the latter one.
--
-- Only non-empty entries from the new msg data will be merged.
function exports.merge_msg(old, new)
    for i, entry in ipairs(new) do
        if entry ~= "" then
            old[i] = entry
        end
    end
end

-- Merges all msgs from a directory.
--
-- The directory must contain files named addresses of msgs to replace, followed by `.msg`.
--
-- The addresses may be either mapped ROM addresses (08XXXXXX) or unmapped file offsets (00XXXXXX): if they are unmapped file offsets, they will be automatically transformed into mapped ROM addresses.
function exports.merge_msgs_from_mod_directory(mpak, dir)
    for _, filename in ipairs(chaudloader.modfiles.list_directory(dir)) do
        local raw_addr = string.match(filename, "^(%x+).msg$")
        if raw_addr == nil then
            goto continue
        end
        local addr = tonumber(raw_addr, 16) | 0x08000000
        exports.edit_msg(mpak, addr, function (msg)
            exports.merge_msg(msg, chaudloader.msg.unpack(chaudloader.modfiles.read_file(dir .. '/' .. filename)))
        end)
        ::continue::
    end
end

return exports
