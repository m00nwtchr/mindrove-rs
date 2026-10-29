mod support;

use mindrove::{Result, syncbox::SyncBox};

fn main() -> Result<()> {
    support::init_tracing();

    let mut syncbox = SyncBox::default();
    syncbox.add_sensor("045188")?;
    syncbox.add_sensor("045000")?;

    syncbox.start_stream()?;
    std::thread::sleep(std::time::Duration::from_secs(5));

    syncbox.stop_stream()?;

    Ok(())
}
