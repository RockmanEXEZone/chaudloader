import os
import shutil
import tarfile
import urllib.request
import subprocess
import glob
from pathlib import Path

lua_version = "5.4.4"

build_dir = Path("build") / "lua54"
build_dir.mkdir(parents = True, exist_ok = True)

os.chdir(build_dir)

lua_dir = Path(f"lua-{lua_version}")
shutil.rmtree(lua_dir, ignore_errors = True)

tar_filename = f"lua-{lua_version}.tar.gz"
url = f"https://www.lua.org/ftp/{tar_filename}"
urllib.request.urlretrieve(url, tar_filename)
with tarfile.open(tar_filename, mode = "r:gz") as tar:
    tar.extractall()
os.remove(tar_filename)

src_dir = lua_dir / "src"
for f in ["lua.c", "luac.c"]:
    (src_dir / f).unlink(missing_ok = True)

subprocess.run([
        "cl",
        "/nologo", "/MD", "/DLUA_BUILD_AS_DLL", "/O2",
        "/c", *src_dir.glob("*.c")
    ],
    check = True
)

obj_files = glob.glob("*.obj")
subprocess.run([
        "link",
        "/nologo", "/DLL", "/IMPLIB:lua54.lib",
        "/OUT:lua54.dll",
        *obj_files
    ],
    check = True
)

include_dir = Path("include")
include_dir.mkdir(parents = True, exist_ok = True)

for h_file in src_dir.glob("*.h"):
    shutil.copy(h_file, include_dir)

shutil.rmtree(lua_dir, ignore_errors = True)
for obj_file in obj_files:
    os.remove(obj_file)
