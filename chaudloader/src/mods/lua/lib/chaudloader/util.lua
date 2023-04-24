-- Unpacks an .map and .mpak for loading, calls a function on it, then writes it back when complete.
local function edit_mpak(dat, name, cb)
    local mpak = chaudloader.Mpak(
        dat:read_file(name .. ".map"),
        dat:read_file(name .. ".mpak")
    )
    cb(mpak)
    local raw_map, raw_mpak = mpak:pack()
    dat:write_file(name .. ".map", raw_map)
    dat:write_file(name .. ".mpak", raw_mpak)
end

-- Unpacks msg data, calls a function on it, then writes it back when complete.
local function edit_msg(mpak, address, cb)
    mpak[address] = chaudloader.pack_msg(cb(chaudloader.unpack_msg(mpak[address])))
end

-- Merges all msgs from a directory.
--
-- - The directory must contain files named addresses of msgs to replace.
-- - Non-empty entries from the source text archive will be merged into the target text archive.
local function merge_msgs_from_mod_directory(mpak, dir)
    for _, filename in ipairs(chaudloader.list_mod_directory(dir)) do
        local raw_addr = string.gmatch(filename, "(%x+).msg$")()
        if raw_addr == nil then
            goto continue
        end
        local addr = tonumber(raw_addr, 16) | 0x08000000
        edit_msg(mpak, addr, function (ta)
            local src_ta = chaudloader.unpack_msg(chaudloader.read_mod_file(dir .. '/' .. filename))
            for i, entry in ipairs(src_ta) do
                if entry ~= "" then
                    ta[i] = entry
                end
            end
            return ta
        end)
        ::continue::
    end
end

return {
    edit_mpak = edit_mpak,
    edit_msg = edit_msg,
    merge_msgs_from_mod_directory = merge_msgs_from_mod_directory,
}
