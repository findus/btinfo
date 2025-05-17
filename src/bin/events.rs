use std::collections::HashMap;
use futures_util::stream::StreamExt;
use std::ffi::OsStr;
use zbus::{proxy, zvariant::OwnedObjectPath, Connection};

#[proxy(
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1",
    interface = "org.freedesktop.systemd1.Manager"
)]
trait Systemd1Manager {
    // Defines signature for D-Bus signal named `JobNew`
    #[zbus(signal)]
    fn unit_new(&self, string: String, path: OwnedObjectPath) -> zbus::Result<()>;
    #[zbus(signal)]
    fn unit_removed(&self, string: String, path: OwnedObjectPath) -> zbus::Result<()>;
}

#[proxy(
    default_service = "org.freedesktop.Notifications",
    default_path = "/fr/emersion/Mako",
    interface = "org.freedesktop.DBus.Properties"
)]
trait Mako {
    #[zbus(signal)]
    fn properties_changed(&self, string: String, val: HashMap<String, zvariant::OwnedValue>, c: Vec<String>) -> zbus::Result<()>;
}

fn notify_waybar() {
    let mut system = sysinfo::System::new();
    system.refresh_all();

    let pid = system
        .processes_by_exact_name(OsStr::new("waybar"))
        .next()
        .map(|e| e.pid().as_u32() as i32);

    if let Some(pid) = pid {
        let signal_number = { libc::SIGRTMIN() + 13 };
        let _ = unsafe { libc::kill(pid, signal_number) };
    } else {
        println!("Waybar not active")
    }
}

async fn watch_systemd_jobs() -> anyhow::Result<()> {
    let connection = Connection::system().await?;
    let session_connection = Connection::session().await?;

    let systemdproxy = Systemd1ManagerProxy::new(&connection).await?;
    let makoproxy = MakoProxy::new(&session_connection).await?;


    let mut new_devices_stream = systemdproxy.receive_unit_new().await?;
    let mut devies_removed_stream = systemdproxy.receive_unit_removed().await?;
    let mut mako_notifications = makoproxy.receive_properties_changed().await?;

    futures_util::try_join!(
        async {
            while let Some(msg) = devies_removed_stream.next().await {
                let args: UnitRemovedArgs = msg.args().expect("Error parsing message");
                if args.string.contains("bluetooth") && args.string.contains("sys-subsystem") {
                    println!("Bluetooth Device removed");
                    notify_waybar();
                }
            }
            Ok::<(), zbus::Error>(())
        },
        async {
            while let Some(msg) = new_devices_stream.next().await {
                let args: UnitNewArgs = msg.args().expect("Error parsing message");
                if args.string.contains("bluetooth") && args.string.contains("sys-subsystem") {
                    println!("Bluetooth Device added");
                    notify_waybar();
                }
            }
            Ok(())
        },
        async {
            while let Some(msg) = mako_notifications.next().await {
                let args: PropertiesChangedArgs = msg.args().expect("Error parsing message");
                println!("we got a mako event!!!!!")
            }
            Ok(())
        }
    )?;

    panic!("Stream ended unexpectedly");
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    watch_systemd_jobs().await
}
