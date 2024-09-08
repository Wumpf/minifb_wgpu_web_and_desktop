use anyhow::Context;

use pico_args::Arguments;
use xshell::Shell;

/// Name of the crate that built & served to the browser.
const CRATE_NAME: &str = "minifb_wgpu_example";

use std::{io, process::Command};

struct Program {
    pub crate_name: &'static str,
    pub binary_name: &'static str,
}

fn check_all_programs(programs: &[Program]) -> anyhow::Result<()> {
    let mut failed_crates = Vec::new();
    for &Program {
        crate_name,
        binary_name,
    } in programs
    {
        let mut cmd = Command::new(binary_name);
        cmd.arg("--help");
        let output = cmd.output();
        match output {
            Ok(_output) => {
                log::info!("Checking for {binary_name} in PATH: ✅");
            }
            Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => {
                log::error!("Checking for {binary_name} in PATH: ❌");
                failed_crates.push(crate_name);
            }
            Err(e) => {
                log::error!("Checking for {binary_name} in PATH: ❌");
                panic!("Unknown IO error: {:?}", e);
            }
        }
    }

    if !failed_crates.is_empty() {
        log::error!(
            "Please install them with: cargo install {}",
            failed_crates.join(" ")
        );

        anyhow::bail!("Missing required programs");
    }

    Ok(())
}

pub fn run_wasm(shell: Shell, mut args: Arguments) -> anyhow::Result<()> {
    let no_serve = args.contains("--no-serve");
    let release = args.contains("--release");

    let programs_needed: &[_] = if no_serve {
        &[Program {
            crate_name: "wasm-bindgen-cli",
            binary_name: "wasm-bindgen",
        }]
    } else {
        &[
            Program {
                crate_name: "wasm-bindgen-cli",
                binary_name: "wasm-bindgen",
            },
            Program {
                crate_name: "simple-http-server",
                binary_name: "simple-http-server",
            },
        ]
    };

    check_all_programs(programs_needed)?;

    let release_flag: &[_] = if release { &["--release"] } else { &[] };
    let output_dir = if release { "release" } else { "debug" };

    log::info!("building the application for wasm");

    let cargo_args = args.finish();

    xshell::cmd!(
        shell,
        "cargo build --target wasm32-unknown-unknown --bin {CRATE_NAME} {release_flag...}"
    )
    .args(&cargo_args)
    .quiet()
    .run()
    .context("Failed to build the application for wasm")?;

    log::info!("running wasm-bindgen");

    xshell::cmd!(
        shell,
        "wasm-bindgen target/wasm32-unknown-unknown/{output_dir}/{CRATE_NAME}.wasm --target web --no-typescript --out-dir target/generated --out-name app"
    )
    .quiet()
    .run()
    .context("Failed to run wasm-bindgen")?;

    // TODO: Run wasm-opt

    let static_files = shell
        .read_dir(format!("{CRATE_NAME}/web_resources"))
        .context("Failed to enumerate static files")?;

    for file in static_files {
        log::info!(
            "copying static file \"{}\"",
            file.canonicalize().unwrap().display()
        );

        shell
            .copy_file(&file, "target/generated")
            .with_context(|| format!("Failed to copy static file \"{}\"", file.display()))?;
    }

    if !no_serve {
        log::info!("serving on port 8000");

        // Explicitly specify the IP address to 127.0.0.1 since otherwise simple-http-server will
        // print http://0.0.0.0:8000 as url which is not a secure context and thus doesn't allow
        // running WebGPU!
        //
        // Disable http caching since it can be excrucingly annoying when developing (changes not showing up at random etc.).
        xshell::cmd!(
            shell,
            "simple-http-server target/generated -c wasm,html,js -i --coep --coop --ip 127.0.0.1 --nocache"
        )
        .quiet()
        .run()
        .context("Failed to simple-http-server")?;
    }

    Ok(())
}
