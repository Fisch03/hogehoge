project_dir := justfile_directory()

plugin_in    := project_dir + "/plugins"
plugin_build := project_dir + "/target"
plugin_out   := project_dir + "/built-plugins"

[working-directory: 'plugin-builder']
_prepare_plugin_builder:
    @echo "Preparing plugin builder..."
    @cargo build 

[working-directory: 'plugin-builder']
_build-plugins: _prepare_plugin_builder
    @echo -e ""
    @cargo -q run -- --in-dir {{plugin_in}} --build-dir {{plugin_build}} --out-dir {{plugin_out}} 
[working-directory: 'plugin-builder']
_build-plugins-release: _prepare_plugin_builder
    @echo -e ""
    @cargo -q run -- --in-dir {{plugin_in}} --build-dir {{plugin_build}} --out-dir {{plugin_out}} --release

run: _build-plugins
    @echo -e "\nBuilding 2hoge..."
    @cargo run -- --plugin-dir {{plugin_out}} 
run-release: _build-plugins-release
    @echo -e "\nBuilding 2hoge..."
    @cargo run --release -- --plugin-dir {{plugin_out}}
    

