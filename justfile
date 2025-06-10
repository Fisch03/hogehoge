project_dir := justfile_directory()

plugin_in    := project_dir + "/plugins"
plugin_build := project_dir + "/target"
plugin_out   := project_dir + "/target/plugins"

theme_in      := project_dir + "/themes"
theme_out     := project_dir + "/target/themes"

[working-directory: 'build/plugin-builder']
_prepare_plugin_builder:
    @echo "Preparing plugin builder..."
    @cargo build --release
[working-directory: 'build/theme-builder']
_prepare_theme_builder:
    @echo "Preparing theme builder..."
    @cargo build 
_prepare_build: _prepare_plugin_builder _prepare_theme_builder

[working-directory: 'build/plugin-builder']
_build-plugins: _prepare_build
    @echo -e ""
    @cargo -q run --release -- --in-dir {{plugin_in}} --build-dir {{plugin_build}} --out-dir {{plugin_out}} 
[working-directory: 'build/plugin-builder']
_build-plugins-release: _prepare_build
    @echo -e ""
    @cargo -q run --release -- --in-dir {{plugin_in}} --build-dir {{plugin_build}} --out-dir {{plugin_out}} --release

[working-directory: 'build/theme-builder']
_build-themes: _prepare_build
    @echo -e ""
    @cargo -q run -- --in-dir {{theme_in}} --out-dir {{theme_out}}

run: _build-plugins _build-themes
    @echo -e "\nBuilding 2hoge..."
    @cargo run -- --plugin-dir {{plugin_out}} --theme-dir {{theme_out}}
run-release: _build-plugins-release _build-themes
    @echo -e "\nBuilding 2hoge..."
    @cargo run --release -- --plugin-dir {{plugin_out}} --theme-dir {{theme_out}}
    

