//! Verify a PNG exported through the native UI against the independently reopened project.
fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let project = args.next().expect("project.ditto");
    let png = args.next().expect("export.png");
    let d = ditto::project::load(std::path::Path::new(&project))?;
    let expected = ditto::project::render_png(&d, 1, None)?;
    let actual = image::open(png)?.into_rgba8();
    anyhow::ensure!(expected == actual, "Export differs from reopened project");
    println!(
        "PNG pixels match reopened project: {} × {}; reference embedded: {}",
        actual.width(),
        actual.height(),
        d.reference.is_some()
    );
    Ok(())
}
