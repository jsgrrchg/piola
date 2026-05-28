use miette::IntoDiagnostic;
use owo_colors::OwoColorize;

pub fn run_update(force: bool) -> miette::Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    println!("{} v{}", "Versión actual:".dimmed(), current.cyan(),);
    println!("{}", "Buscando actualizaciones...".dimmed());

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

    if status.updated() {
        println!(
            "{} Actualizado a v{}",
            "✓".green().bold(),
            status.version().cyan().bold(),
        );
    } else {
        println!(
            "{} Wena choro! Ya tienes la última versión (v{}).",
            "✓".green(),
            current.cyan(),
        );
    }

    Ok(())
}
