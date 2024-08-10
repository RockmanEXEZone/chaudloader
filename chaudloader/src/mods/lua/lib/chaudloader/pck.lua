-- Replaces wem file language ID SFX
function chaudloader.pck.replace_wem_sfx(id, path)
    chaudloader.pck.replace_wem(id, path, 0)
end
-- Replaces wem file language ID Japanese
function chaudloader.pck.replace_wem_japanese(id, path)
    chaudloader.pck.replace_wem(id, path, 1)
end
-- Replaces wem file language ID Chinese
function chaudloader.pck.replace_wem_chinese(id, path)
    chaudloader.pck.replace_wem(id, path, 2)
end
-- Replaces wem file language ID English
function chaudloader.pck.replace_wem_english(id, path)
    chaudloader.pck.replace_wem(id, path, 3)
end
