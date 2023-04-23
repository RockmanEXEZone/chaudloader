$lua_version="5.4.4"

New-Item -ItemType Directory -ErrorAction silentlycontinue build
New-Item -ItemType Directory -ErrorAction silentlycontinue build/lua54
Push-Location build/lua54
Remove-Item -ErrorAction silentlycontinue -Recurse "lua-${lua_version}"
Invoke-WebRequest -Uri "https://www.lua.org/ftp/lua-${lua_version}.tar.gz" -OutFile "lua-${lua_version}.tar.gz"
tar -xzvf "lua-${lua_version}.tar.gz"
Remove-Item "lua-${lua_version}.tar.gz"
Remove-Item "lua-${lua_version}/src/lua.c","lua-${lua_version}/src/luac.c"
cl /nologo /MD /DLUA_BUILD_AS_DLL /O2 /c "lua-${lua_version}/src/*.c"
link /nologo /DLL /IMPLIB:lua54.lib /OUT:lua54.dll *.obj
New-Item -ItemType Directory -ErrorAction silentlycontinue include
Copy-Item "lua-${lua_version}/src/*.h" include/
Remove-Item -Recurse "lua-${lua_version}"
Remove-Item *.obj
Pop-Location
