struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) color: vec4<f32>, @location(2) @interpolate(flat) alpha_filter: f32 };
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
override SURFACE_SRGB: f32 = 0.0;
@vertex fn vs_main(@location(0) position:vec2<f32>, @location(1) uv:vec2<f32>, @location(2) color:vec4<f32>, @location(3) alpha_filter:f32) -> VertexOut {
  var out:VertexOut;out.position=vec4<f32>(position,0.0,1.0);out.uv=uv;out.color=color;out.alpha_filter=alpha_filter;return out;
}
fn premultiplied_texel(p: vec2<i32>, size: vec2<i32>) -> vec4<f32> {
  let c=textureLoad(tex,clamp(p,vec2(0),size-vec2(1)),0);
  return vec4(c.rgb*c.a,c.a);
}
fn artwork_sample(uv: vec2<f32>) -> vec4<f32> {
  let size=vec2<i32>(textureDimensions(tex));
  let pos=uv*vec2<f32>(size)-vec2(0.5);
  let p=vec2<i32>(floor(pos));
  let f=fract(pos);
  let top=mix(premultiplied_texel(p,size),premultiplied_texel(p+vec2(1,0),size),f.x);
  let bottom=mix(premultiplied_texel(p+vec2(0,1),size),premultiplied_texel(p+vec2(1,1),size),f.x);
  let c=mix(top,bottom,f.y);
  if c.a <= 0.00001 { return vec4(0.); }
  return vec4(c.rgb/c.a,c.a);
}
@fragment fn fs_main(in:VertexOut)->@location(0) vec4<f32> {
  var sampled=textureSample(tex,smp,in.uv);
  // Artwork has straight-alpha colours, unlike the white glyph atlas. Filter
  // in premultiplied space so empty texels cannot darken antialiased edges.
  if in.alpha_filter > 0.5 { sampled=artwork_sample(in.uv); }
  var color=sampled*in.color;
  if SURFACE_SRGB > 0.5 { color=vec4<f32>(select(color.rgb/12.92,pow((color.rgb+0.055)/1.055,vec3<f32>(2.4)),color.rgb>vec3<f32>(0.04045)),color.a); }
  return color;
}
