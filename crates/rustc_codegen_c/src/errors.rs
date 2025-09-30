use rustc_macros::Diagnostic;

#[derive(Diagnostic)]
#[diag(codegen_c_c_file_write_err)]
pub(crate) struct CFileWriteError {
    pub err: std::io::Error,
}

#[derive(Diagnostic)]
#[diag(codegen_c_cc_spawn_err)]
pub(crate) struct CCompilerSpawnError {
    pub err: std::io::Error,
}

#[derive(Diagnostic)]
#[diag(codegen_c_cc_err)]
pub(crate) struct CCompilerError {
    pub cc_stderr: String,
    pub cc_stdout: String,
}
