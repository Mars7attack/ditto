struct Settings {
    // operation, blend, pixel scale, blur pass radius in document pixels
    control: vec4<f32>,
    params: vec4<f32>,
    dark: vec4<f32>,
    light: vec4<f32>,
};
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var original: texture_2d<f32>;
@group(0) @binding(2) var<uniform> settings: Settings;

@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let points = array<vec2<f32>, 3>(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.));
    return vec4(points[i], 0., 1.);
}
fn read(p: vec2<i32>) -> vec4<f32> {
    let size = vec2<i32>(textureDimensions(source));
    if any(p < vec2(0)) || any(p >= size) { return vec4(0.); }
    return textureLoad(source, p, 0);
}
fn premul(c: vec4<f32>) -> vec4<f32> { return vec4(c.rgb * c.a, c.a); }
fn straight(c: vec4<f32>) -> vec4<f32> {
    if c.a < 0.00001 { return vec4(0.); }
    return vec4(clamp(c.rgb / c.a, vec3(0.), vec3(1.)), clamp(c.a, 0., 1.));
}
fn luma(c: vec3<f32>) -> f32 { return dot(c, vec3(0.2126, 0.7152, 0.0722)); }
fn hue_rotate(c: vec3<f32>, angle: f32) -> vec3<f32> {
    // YIQ rotation preserves luminance and makes the 0-degree setting neutral.
    let y = dot(c, vec3(0.299, 0.587, 0.114));
    let i = dot(c, vec3(0.596, -0.274, -0.322));
    let q = dot(c, vec3(0.211, -0.523, 0.312));
    let a = radians(angle);
    let ri = i * cos(a) - q * sin(a);
    let rq = i * sin(a) + q * cos(a);
    return vec3(y + 0.956 * ri + 0.621 * rq, y - 0.272 * ri - 0.647 * rq, y - 1.106 * ri + 1.703 * rq);
}
@fragment fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pos = vec2<i32>(position.xy);
    let size = vec2<f32>(textureDimensions(source));
    let uv = position.xy / size;
    let scale = settings.control.z;
    let xy = position.xy / scale;
    let op = u32(settings.control.x);
    let p = settings.params.xyz;
    let c = read(pos);
    // Glow: threshold and horizontal blur, vertical blur, then composite.
    // Intermediate glow textures store premultiplied light, so transparency never creates fringes.
    if op == 9u || op == 10u || op == 13u || op == 14u {
        var sum = vec4(0.);
        var weights = 0.;
        // Sample every texel: a fixed 25-tap kernel skips thin strokes at 4x
        // and produces a checkerboard halo instead of a continuous blur.
        let blur_radius = select(p.x, settings.control.w, op >= 13u);
        let radius = i32(ceil(blur_radius * scale));
        let sigma = max(blur_radius * scale * (5. / 12.), 0.5);
        for (var tap = -radius; tap <= radius; tap += 1) {
            let weight = exp(-0.5 * pow(f32(tap) / sigma, 2.));
            var shift = vec2<i32>(tap, 0);
            if op == 10u || op == 14u { shift = shift.yx; }
            var sample = read(pos + shift);
            if op == 9u {
                let brightness = luma(sample.rgb);
                let gate = smoothstep(p.z, min(p.z + 0.15, 1.001), brightness);
                sample = vec4(sample.rgb * sample.a * gate, sample.a * gate);
            } else if op == 13u {
                sample = premul(sample);
            }
            sum += sample * weight;
            weights += weight;
        }
        return sum / weights;
    }
    var base = c;
    var result = c;
    switch op {
        case 0u: {
            base = textureLoad(original, pos, 0);
            let glow = c * p.y;
            let alpha = base.a + clamp(glow.a, 0., 1.) * (1. - base.a);
            let base_light = base.rgb * base.a;
            let halo = clamp(glow.rgb, vec3(0.), vec3(1.));
            result = straight(vec4(base_light + halo * (vec3(1.) - base_light), alpha));
        }
        case 1u: {
            let band = uv.x * 0.75 + uv.y * 0.25;
            let highlight = exp(-pow((band - p.x) / p.y, 2.) * 3.) * p.z;
            result = vec4(c.rgb + vec3(highlight), c.a);
        }
        case 2u: {
            let rotated = hue_rotate(c.rgb, p.x);
            result = vec4(mix(vec3(luma(rotated)), rotated, p.y) * exp2(p.z), c.a);
        }
        case 3u: {
            var value = clamp((luma(c.rgb) - p.y - 0.5) * p.x + 0.5, 0., 1.);
            value = mix(value, 1. - value, p.z);
            result = vec4(mix(settings.dark.rgb, settings.light.rgb, value), c.a);
        }
        case 4u: {
            let line = 1. - smoothstep(p.y - 0.06, p.y + 0.06, fract(xy.y / p.x));
            result = vec4(c.rgb * (1. - line * p.z), c.a);
        }
        case 5u: {
            let tile = fract(xy / p.x) - vec2(0.5);
            var mask = 1. - smoothstep(p.y * 0.65 - 0.05, p.y * 0.65 + 0.05, length(tile));
            if p.z > 0.5 && p.z < 1.5 { mask = 1. - smoothstep(p.y - 0.05, p.y + 0.05, fract((xy.x + xy.y) / p.x)); }
            if p.z >= 1.5 {
                mask = select(p.y, 1., (u32(floor(xy.x / p.x)) + u32(floor(xy.y / p.x))) % 2u == 0u);
            }
            result = vec4(c.rgb * (0.2 + 0.8 * mask), c.a);
        }
        case 6u: {
            let offset = vec2(cos(radians(p.y)), sin(radians(p.y))) * p.x * scale;
            let red = premul(read(pos + vec2<i32>(round(offset * (0.5 + p.z)))));
            let blue = premul(read(pos - vec2<i32>(round(offset * (1.5 - p.z)))));
            let alpha = max(c.a, max(red.a, blue.a));
            result = straight(vec4(red.r, c.g * c.a, blue.b, alpha));
        }
        case 7u: {
            // Integer noise is stable across GPUs and independent of output resolution.
            let tile = vec2<u32>(floor(xy / p.x));
            var hash = tile.x * 1973u + tile.y * 9277u + u32(p.z) * 26699u + 17u;
            hash = (hash ^ (hash >> 13u)) * 1274126177u;
            hash = hash ^ (hash >> 16u);
            let noise = f32(hash & 65535u) / 65535. - 0.5;
            result = vec4(c.rgb + vec3(noise * p.y), c.a);
        }
        case 8u: {
            let distance = length((uv - vec2(0.5)) * 1.41421356);
            let vignette = smoothstep(p.x * (1. - p.y), p.x, distance);
            result = vec4(c.rgb * (1. - vignette * p.z), c.a);
        }
        case 11u: {
            base = textureLoad(original, pos, 0);
            result = straight(c);
        }
        case 12u: {
            base = textureLoad(original, pos, 0);
            // Detect transitions in the smoothed neighbourhood. Using the raw
            // centre-minus-blur difference would leave sharp rings at antialiased
            // contours; this continuous mask keeps the softened edge uniform.
            var low = c;
            var high = c;
            let distance = max(1, i32(round(p.x * scale)));
            for (var y = -1; y <= 1; y += 1) {
                for (var x = -1; x <= 1; x += 1) {
                    let neighbour = read(pos + vec2(x, y) * distance);
                    low = min(low, neighbour);
                    high = max(high, neighbour);
                }
            }
            let spread = high - low;
            let contrast = max(max(spread.r, spread.g), max(spread.b, spread.a));
            let edge = smoothstep(p.y, p.y + p.z, contrast);
            result = straight(mix(premul(base), c, edge));
        }
        default: {}
    }
    result = clamp(result, vec4(0.), vec4(1.));
    return straight(mix(premul(base), premul(result), settings.control.y));
}
