fn main() {
    let lua_dll_path = std::path::Path::new("../build/lua54/lua54.dll");

    if lua_dll_path.exists() {
        let _ = omnicopy_to_output::cargo_rerun_if_path_changed(lua_dll_path);
        omnicopy_to_output::copy_to_output_by_path(lua_dll_path).expect("Could not copy");
    }
}
