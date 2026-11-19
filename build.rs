fn main() -> Result<(), slint_build::CompileError> {
    let configuration = slint_build::CompilerConfiguration::new().with_style("fluent-dark".into());
    slint_build::compile_with_config("ui/app-window.slint", configuration)
}
