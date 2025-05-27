// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use libsui::{Elf, Macho, PortableExecutable};
use std::error::Error;
use std::fs::File;
use std::{env::current_exe, path::Path};

pub static ANDROMEDA_JS_CODE_SECTION: &str = "ANDROMEDABINCODE";

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
        Elf::new(&exe).append(ANDROMEDA_JS_CODE_SECTION, &js, &mut out)?;
    } else if os == "windows" {
        PortableExecutable::from(&exe)?
            .write_resource(ANDROMEDA_JS_CODE_SECTION, js)?
            .build(&mut out)?;
    } else {
        return Err("Unsupported operating system".into());
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::fs::{metadata, set_permissions};
        use std::os::unix::fs::PermissionsExt;

        // Make the binary executable on Unix-like systems
        if os == "macos" || os == "linux" {
            let mut perms = metadata(result_name)?.permissions();
            perms.set_mode(0o755); // rwxr-xr-x permissions
            set_permissions(result_name, perms)?;
        }
    }

    Ok(())
}
