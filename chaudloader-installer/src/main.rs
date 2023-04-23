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

const FILES_TO_COPY: &[&str] = &["dxgi.dll", "chaudloader.dll", "lua54.dll"];
const FILES_TO_DELETE: &[&str] = &["bnlc_mod_loader.dll"];

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
            println!("It is possible that the installer was not able to detect your installation automatically. If this is the case, please copy the following files into the same directory as MMBN_LC1.exe and MMBN_LC2.exe:");
            for filename in FILES_TO_COPY.iter() {
                println!(" - {}", filename);
            }
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

        let files = FILES_TO_COPY
            .iter()
            .map(|filename| {
                Ok::<_, anyhow::Error>((filename, std::fs::read(src_path.join(filename))?))
            })
            .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?;

        for path in paths {
            let exe_path = path.join("exe");

            for filename in FILES_TO_DELETE.iter() {
                let path = exe_path.join(filename);
                match std::fs::remove_file(&path) {
                    Ok(()) => {
                        println!("DELETE  {}", path.display());
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }

            for (filename, contents) in files.iter() {
                let path = exe_path.join(filename);
                let mut f = std::fs::File::create(&path)?;
                f.write_all(&contents)?;
                println!("COPY   {}", path.display());
            }

            let mods_path = exe_path.join("mods");
            match std::fs::create_dir(&mods_path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => {
                    return Err(e.into());
                }
            }
            println!("MKDIR  {}", mods_path.display());
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
