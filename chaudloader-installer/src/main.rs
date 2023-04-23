use std::io::Write;

use clap::Parser;

const BANNER: &str = "
        %%%%%%%%%%%%%%%%%
     %%%%%  *********  %%%%%
   %%%% *************     %%%%
  %%% ***************       %%%
 %%% *************** ******* %%%
 %%% ************ ********** %%%    chaudloader
 %%% ********** ************ %%%    Installer
 %%% ******* *************** %%%
  %%%       *************** %%%
   %%%%     ************* %%%%
     %%%%%  *********  %%%%%
        %%%%%%%%%%%%%%%%%
";

#[derive(clap::Parser)]
struct Args {
    /// Skip confirmations and run non-interactively.
    #[arg(short, long)]
    yes: bool,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let r = (|| {
        println!("{}", BANNER);

        println!("Welcome to the chaudloader installer.");
        println!();

        let mut steamdir = steamlocate::SteamDir::locate()
            .ok_or_else(|| anyhow::anyhow!("could not initialize steam dir"))?;

        let apps = steamdir.apps();

        let mut paths = vec![];

        println!("Found games:");
        if let Some(app) = apps.get(&1798010).and_then(|v| v.as_ref()) {
            println!(" - Vol. 1: {}", app.path.display());
            paths.push(app.path.clone())
        }
        if let Some(app) = apps.get(&1798020).and_then(|v| v.as_ref()) {
            println!(" - Vol. 2: {}", app.path.display());
            paths.push(app.path.clone())
        }

        if paths.is_empty() {
            println!(
                " ! Mega Man Battle Network Legacy Collection could not be detected on your computer."
            );
            println!();
            println!("Installation cancelled.");
            return Ok(());
        }

        if !args.yes {
            print!("Do you wish to proceed? [Y/n] ");
            std::io::stdout().flush()?;
            let mut response = String::new();
            std::io::stdin().read_line(&mut response)?;
            response = response.trim().to_lowercase();
            if response != "y" && response != "" {
                println!("Installation cancelled.");
                return Ok(());
            }
        }

        println!();

        let exe_path = std::env::current_exe()?;
        let src_path = exe_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("could not get parent directory"))?;

        let dxgi_dll = std::fs::read(src_path.join("dxgi.dll"))?;
        let chaudloader_dll = std::fs::read(src_path.join("chaudloader.dll"))?;

        for path in paths {
            let exe_path = path.join("exe");

            let dxgi_dll_path = exe_path.join("dxgi.dll");
            let mut dxgi_dll_f = std::fs::File::create(&dxgi_dll_path)?;
            dxgi_dll_f.write_all(&dxgi_dll)?;
            println!("OK: {}", dxgi_dll_path.display());

            let chaudloader_dll_path = exe_path.join("chaudloader.dll");
            let mut chaudloader_dll_f = std::fs::File::create(&chaudloader_dll_path)?;
            chaudloader_dll_f.write_all(&chaudloader_dll)?;
            println!("OK: {}", chaudloader_dll_path.display());

            let mods_path = exe_path.join("mods");
            match std::fs::create_dir(&mods_path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => {
                    return Err(e.into());
                }
            }
            println!("OK: {}", mods_path.display());
        }
        println!();

        println!("Installation successful!");
        Ok(())
    })();

    if let Err(err) = &r {
        println!("Installation failed with error: {}", err);
    }

    if !args.yes {
        println!();
        print!("Press enter or close this window to finish.");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }

    r
}
