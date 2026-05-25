use miette::IntoDiagnostic;

pub fn run_update(force: bool) -> miette::Result<()> {
    let current = env!("CARGO_PKG_VERSION");
    println!("Versión actual: v{current}");
    println!("Buscando actualizaciones...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner("cuervolu")
        .repo_name("wn")
        .identifier("wn-cli")
        .bin_name("wn")
        .show_output(false)
        .show_download_progress(true)
        .current_version(current)
        .no_confirm(force)
        .build()
        .into_diagnostic()?
        .update()
        .into_diagnostic()?;

    match status.updated() {
        true => println!("✓ Actualizado a v{}", status.version()),
        false => println!("Ya tienes la última versión (v{current})."),
    }

    Ok(())
}
