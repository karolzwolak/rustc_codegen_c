use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use rustc_codegen_ssa::back::write::{CodegenContext, ModuleConfig};
use rustc_codegen_ssa::{CompiledModule, ModuleCodegen};
use rustc_errors::{DiagCtxtHandle, FatalError};
use rustc_session::config::OutputType;
use tracing::error;

fn write_module(c_out: &PathBuf, module: &ModuleCodegen<String>) -> io::Result<()> {
    let mut c_out_file = fs::File::create(c_out)?;
    writeln!(c_out_file, "// file: {}.c", module.name)?;
    write!(c_out_file, "{}", module.module_llvm)?;
    Ok(())
}
fn compiled_module(
    success: bool,
    module: ModuleCodegen<String>,
    cgcx: &CodegenContext<crate::CCodegen>,
) -> CompiledModule {
    module.into_compiled_module(
        success,
        false,
        false,
        false,
        false,
        &cgcx.output_filenames,
        cgcx.invocation_temp.as_deref(),
    )
}

pub(crate) fn codegen(
    cgcx: &CodegenContext<crate::CCodegen>,
    module: ModuleCodegen<String>,
    _config: &ModuleConfig,
) -> CompiledModule {
    let dcx = cgcx.create_dcx();
    let dcx = dcx.handle();
    let obj_out = cgcx.output_filenames.temp_path_for_cgu(
        OutputType::Object,
        &module.name,
        cgcx.invocation_temp.as_deref(),
    );
    let c_out = obj_out.with_extension("c");

    // output c source code
    if let Err(c_out_err) = write_module(&c_out, &module) {
        dcx.emit_err(crate::errors::CFileWriteError { err: c_out_err });
        return compiled_module(false, module, cgcx);
    }

    // invoke cc to compile
    // FIXME: configure cc
    // FIXME: handle long command line (windows)
    // FIXME: flush_linked_file (windows)
    let mut cmd = Command::new("clang");
    cmd.arg(&c_out).arg("-o").arg(&obj_out).arg("-c");
    let success = match cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|child| child.wait_with_output())
    {
        Ok(output) if !output.status.success() => {
            error!("compiler stderr:\n{}", String::from_utf8_lossy(&output.stderr));
            error!("compiler stdout:\n{}", String::from_utf8_lossy(&output.stdout));
            dcx.emit_err(crate::errors::CCompilerError {
                cc_stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                cc_stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            });
            false
        }
        Ok(_) => true,
        Err(e) => {
            error!("failed to spawn C compiler: {}", e);
            dcx.emit_err(crate::errors::CCompilerSpawnError { err: e });
            false
        }
    };

    compiled_module(success, module, cgcx)
}

pub(crate) fn link(
    _cgcx: &CodegenContext<crate::CCodegen>,
    _dcx: DiagCtxtHandle<'_>,
    mut _modules: Vec<ModuleCodegen<String>>,
) -> Result<ModuleCodegen<String>, FatalError> {
    unimplemented!();
}
