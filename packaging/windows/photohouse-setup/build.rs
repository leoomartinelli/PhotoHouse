fn main() -> Result<(), Box<dyn std::error::Error>> {
    // winresource emits self-contained RC with numeric constants and no includes/macros.
    // Feed it unchanged to windres; the Rust GNU toolchain need not include GCC's cc1.
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.iter().any(|a| a == "--copy-resource") {
        let path = args.last().ok_or("Resource path missing")?;
        let mut file = std::fs::File::open(path)?;
        std::io::copy(&mut file, &mut std::io::stdout())?;
        return Ok(());
    }
    println!("cargo:rerun-if-env-changed=PHOTOHOUSE_MSI");
    let msi = std::env::var("PHOTOHOUSE_MSI")?;
    println!("cargo:rerun-if-changed={msi}");
    println!("cargo:rerun-if-changed=../photohouse.ico");
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../photohouse.ico")
        .set("ProductName", "PhotoHouse")
        .set("FileDescription", "Instalador do PhotoHouse")
        .set("OriginalFilename", "PhotoHouse-Setup.exe")
        .set_manifest(r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0"><trustInfo xmlns="urn:schemas-microsoft-com:asm.v3"><security><requestedPrivileges><requestedExecutionLevel level="asInvoker" uiAccess="false"/></requestedPrivileges></security></trustInfo><compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1"><application><supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/></application></compatibility></assembly>"#);
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR missing")?);
        let rc = out.join("resource.rc");
        let object = out.join("resource.o");
        res.write_resource_file(&rc)?;
        let status = std::process::Command::new("windres")
            .arg(format!("--preprocessor=\"{}\" --copy-resource", std::env::current_exe()?.display()))
            .arg("--target=pe-x86-64")
            .arg(&rc).arg(&object).status()?;
        if !status.success() { return Err("Windows resource compilation failed".into()); }
        println!("cargo:rustc-link-arg={}", object.display());
    } else {
        res.compile()?;
    }
    Ok(())
}
