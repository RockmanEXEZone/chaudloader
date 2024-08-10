-- Compatibility shims.
-- 0.1 -> 0.8
bnlc_mod_loader = {}

function bnlc_mod_loader.write_exe_dat_contents(dat_filename, path, contents)
    chaudloader.exedat.open(dat_filename):write_file(path, contents)
end

function bnlc_mod_loader.read_exe_dat_contents(dat_filename, path)
    return chaudloader.exedat.open(dat_filename):read_file(path)
end

function bnlc_mod_loader.read_mod_contents(path)
    return chaudloader.modfiles.read_file(path)
end

-- 0.7 -> 0.8
chaudloader.ExeDat = chaudloader.exedat.open
chaudloader.Mpak = chaudloader.mpak.unpack
chaudloader.unpack_msg = chaudloader.msg.unpack
chaudloader.pack_msg = chaudloader.msg.pack
chaudloader.read_mod_file = chaudloader.modfiles.read_file
chaudloader.list_mod_directory = chaudloader.modfiles.list_directory
chaudloader.get_mod_file_metadata = chaudloader.modfiles.get_file_metadata
