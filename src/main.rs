use std::process::Command;

use clap::Parser;
use notify_rust::Notification;

mod package;
use package::get_package_version;

mod kernel;
use kernel::KernelInfo;

/// Parse the output of `xdpyinfo`
fn parse_xdpyinfo_output(xdpyinfo_output: &str) -> Option<&str> {
    for line in xdpyinfo_output.lines() {
        if line.starts_with("X.Org version: ") {
            return Some(line.split_at(15).1);
        }
    }
    None
}

#[derive(Debug, Parser)]
#[clap(
    version,
    about = "Check the currently installed kernel against the currently running one."
)]
struct Args {
    /// Disable desktop notification
    #[structopt(long)]
    disable_notification: bool,
}

fn main() {
    let args = Args::from_args();

    // Initialize Pacman database
    let alpm = alpm::Alpm::new("/", "/var/lib/pacman/")
        .expect("Could not open pacman database at /var/lib/pacman");
    let db = alpm.localdb();

    // uname output is in the form version-ARCH
    let kernel_info = KernelInfo::from_uname().expect("Failed to parse uname output");
    let running_kernel_version = kernel_info.version;

    let kernel_package = if let Some(variant) = kernel_info.variant {
        format!("linux-{}", variant)
    } else {
        "linux".to_owned()
    };

    let installed_kernel = get_package_version(&db, &kernel_package)
        .expect("Could not get version of installed kernel");

    println!("Kernel");
    println!(
        " installed: {} (since {})",
        installed_kernel.version,
        installed_kernel.installed_reltime()
    );
    println!(" running:   {}", running_kernel_version);

    let should_reboot = !installed_kernel.version_matches(&running_kernel_version);
    if should_reboot {
        println!("You should reboot arch btw!");
        if !args.disable_notification {
            Notification::new()
                .summary("Reboot arch btw")
                .body("Kernel got updated. You should reboot your system!")
                .timeout(6000) //milliseconds
                .show()
                .map_err(|e| println!("Couldn't send notification: {}", e))
                .ok();
        }
    }

    let output_xdpyinfo = Command::new("xdpyinfo")
        .output()
        .map_err(|err| println!("Could not execute xdpyinfo: {}", err));
    if let Ok(output_xdpyinfo) = output_xdpyinfo {
        let output_xdpyinfo = String::from_utf8_lossy(&output_xdpyinfo.stdout);
        let running_xorg_version =
            parse_xdpyinfo_output(&output_xdpyinfo).expect("Could not parse xdpyinfo output");
        let installed_xorg = get_package_version(&db, "xorg-server")
            .expect("Could not get version of installed xserver");

        println!("Xorg server");
        println!(
            " installed: {} (since {})",
            installed_xorg.version,
            installed_xorg.installed_reltime()
        );
        println!(" running:   {}", running_xorg_version);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_xdpyinfo_output() {
        assert_eq!(
            Some("1.18.4"),
            parse_xdpyinfo_output("X.Org version: 1.18.4")
        );
    }
}
