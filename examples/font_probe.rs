fn main() {
    for path in [
        "/System/Library/Fonts/SFNSMono.ttf",
        "/Library/Fonts/SF-Mono-Regular.otf",
    ] {
        let data = std::fs::read(path).unwrap();
        match fontdue::Font::from_bytes(data, fontdue::FontSettings::default()) {
            Ok(f) => {
                let px = 8. / f.metrics('M', 1.).advance_width;
                println!(
                    "{path}: {:?} px={px} lines={:?}",
                    f.name(),
                    f.horizontal_line_metrics(px)
                );
                let mut count = 0;
                for c in ditto::font::CHARS.iter() {
                    if f.lookup_glyph_index(*c) != 0 {
                        count += 1;
                    }
                }
                println!("coverage {count}/{}", ditto::font::CHARS.len());
                for c in "AgéÉpq░▒▓█▖▗⣿🬀".chars() {
                    let (m, _) = f.rasterize(c, px);
                    println!("{c} index={} metrics={m:?}", f.lookup_glyph_index(c));
                }
            }
            Err(e) => println!("Error {e}"),
        }
    }
}
