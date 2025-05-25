use std::error::Error;
use std::{env::current_exe, path::Path};
use std::fs::File;
use libsui::{Macho, Elf, PortableExecutable};

pub static ANDROMEDA_JS_CODE_SECTION: &'static str = "ANDROMEDABINCODE";

pub fn compile(result_name: &Path, input_file: &Path) -> Result<(), Box<dyn Error>> {
    let exe_path = current_exe()?;
    let exe = std::fs::read(exe_path)?;
    let js = std::fs::read(input_file)?;
    let mut out = File::create(result_name)?;

    // TODO(lino-levan): Replace this with a flag in the CLI
    let os = std::env::consts::OS;

    if os == "macos" {
        Macho::from(exe)?
            .write_section(ANDROMEDA_JS_CODE_SECTION, js)?
            .build_and_sign(&mut out)?;
    } else if os == "linux" {
        Elf::new(&exe)
            .append(ANDROMEDA_JS_CODE_SECTION, &js, &mut out)?;
    } else if os == "windows" {
        PortableExecutable::from(&exe)?
            .write_resource(ANDROMEDA_JS_CODE_SECTION, js)?
            .build(&mut out)?;
    } else {
        return Err("Unsupported operating system".into());
    }

    Ok(())
}
