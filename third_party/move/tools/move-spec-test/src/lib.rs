pub mod cli;
mod prover;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crate::prover::prove;
use anyhow::anyhow;
use move_package::BuildConfig;
use std::path::PathBuf;

/// This function runs the specification testing, which is a combination of the
/// mutator tool and the prover tool
/// It takes the CLI options and constructs appropriate options for the
/// Move Mutator tool and Move Prover tool. Then it mutates the code storing
/// results in a temporary directory. Then it runs the prover on the mutated
/// code and remember the results, using them to generate the report at the end.
///
/// # Arguments
///
/// * `options` - A `cli::Options` representing the options for the spec test.
/// * `config` - A `BuildConfig` representing the build configuration.
/// * `package_path` - A `PathBuf` representing the path to the package.
///
/// # Returns
///
/// * `anyhow::Result<()>` - The result of the spec test.
pub fn run_spec_test(
    options: cli::Options,
    config: BuildConfig,
    package_path: PathBuf,
) -> anyhow::Result<()> {
    // We need to initialize logger using try_init() as it might be already initialized in some other tool
    // (e.g. spec-test). If we use init() instead, we will get an abort.
    let _ = pretty_env_logger::try_init();

    info!("Running spec test");

    let mut mutator_conf = cli::create_mutator_options(&options);
    let prover_conf = cli::generate_prover_options(&options)?;

    // Setup temporary directory structure
    let outdir = tempfile::tempdir()?.into_path();
    let outdir_mutant = outdir.join("mutants");
    let outdir_original = outdir.join("base");

    std::fs::create_dir_all(&outdir_mutant)?;
    std::fs::create_dir_all(&outdir_original)?;

    mutator_conf.out_mutant_dir = outdir_mutant.clone();

    debug!("Running the move mutator tool");

    move_mutator::run_move_mutator(mutator_conf, config.clone(), package_path.clone())?;

    let report =
        move_mutator::report::Report::load_from_json_file(&outdir_mutant.join("report.json"))?;

    // Proving part
    move_mutator::compiler::copy_dir_all(&package_path, &outdir_original)?;

    let mut error_writer = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);

    let result = prove(&config, &package_path, &prover_conf, &mut error_writer);

    if let Err(e) = result {
        let msg = format!(
            "Original code verification failed! Prover failed with error: {}",
            e
        );
        error!("{msg}");
        return Err(anyhow!(msg));
    }

    // TODO: change this to report generation
    let mut total_mutants = 0;
    let mut killed_mutants = 0;

    for elem in report.get_mutants() {
        total_mutants += 1;

        let result = prover::prove_mutant(
            &config,
            &elem.get_mutant_path(),
            &elem.get_original_file_path(),
            &package_path,
            &prover_conf,
            &outdir.join("prove"),
            &mut error_writer,
        );

        if let Err(e) = result {
            trace!("Mutant killed! Prover failed with error: {}", e);
            killed_mutants += 1;
        } else {
            trace!("Mutant hasn't been killed!");
        }
    }

    println!("Total mutants: {}", total_mutants);
    println!("Killed mutants: {}", killed_mutants);

    Ok(())
}
