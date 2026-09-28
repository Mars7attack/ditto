#!/usr/bin/env swift
// Render real macOS font fallback, rather than Ditto's atlas. Read-only inputs.
// swift scripts/probe-text-layout.swift INPUT.txt OUTPUT_PREFIX [FONT] [SIZE]
import AppKit
import CoreText

let args = CommandLine.arguments
guard args.count >= 3 else {
    fputs("Usage: probe-text-layout.swift INPUT.txt OUTPUT_PREFIX [FONT] [SIZE]\n", stderr)
    exit(2)
}
let input = try String(contentsOfFile: args[1], encoding: .utf8)
let output = args[2]
let fontName = args.count > 3 ? args[3] : "Menlo"
let fontSize = args.count > 4 ? Double(args[4])! : 22
let font = CTFontCreateWithName(fontName as CFString, fontSize, nil)
let attributes: [NSAttributedString.Key: Any] = [
    NSAttributedString.Key(kCTFontAttributeName as String): font,
    NSAttributedString.Key(kCTForegroundColorAttributeName as String): NSColor.white.cgColor,
]
let strings = input.components(separatedBy: "\n")
let lines = strings.map { CTLineCreateWithAttributedString(NSAttributedString(string: $0, attributes: attributes)) }
let advanceY = ceil(CTFontGetAscent(font) + CTFontGetDescent(font) + CTFontGetLeading(font))
let width = Int(ceil(lines.map { CTLineGetTypographicBounds($0, nil, nil, nil) }.max() ?? 0)) + 40
let height = Int(ceil(advanceY * Double(lines.count))) + 40
guard let ctx = CGContext(data: nil, width: width, height: height, bitsPerComponent: 8, bytesPerRow: width * 4,
                          space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else {
    fatalError("Cannot allocate render target")
}
ctx.setFillColor(CGColor(gray: 0.118, alpha: 1))
ctx.fill(CGRect(x: 0, y: 0, width: width, height: height))
var glyphs = [[String: Any]]()
for (row, line) in lines.enumerated() {
    ctx.textPosition = CGPoint(x: 20, y: Double(height) - 20 - CTFontGetAscent(font) - Double(row) * advanceY)
    CTLineDraw(line, ctx)
    for run in CTLineGetGlyphRuns(line) as! [CTRun] {
        let attrs = CTRunGetAttributes(run) as NSDictionary
        let runFont = attrs[kCTFontAttributeName] as! CTFont
        let count = CTRunGetGlyphCount(run)
        var positions = [CGPoint](repeating: .zero, count: count)
        var advances = [CGSize](repeating: .zero, count: count)
        var indices = [CFIndex](repeating: 0, count: count)
        CTRunGetPositions(run, CFRange(location: 0, length: 0), &positions)
        CTRunGetAdvances(run, CFRange(location: 0, length: 0), &advances)
        CTRunGetStringIndices(run, CFRange(location: 0, length: 0), &indices)
        let text = strings[row] as NSString
        for i in 0..<count {
            glyphs.append(["row": row, "utf16_index": indices[i], "codepoint": Int(text.character(at: indices[i])),
                           "font": CTFontCopyPostScriptName(runFont) as String, "x": positions[i].x,
                           "advance": advances[i].width])
        }
    }
}
let image = NSBitmapImageRep(cgImage: ctx.makeImage()!)
try image.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: output + ".png"))
let report: [String: Any] = ["requested_font": fontName, "font_size": fontSize, "line_advance": advanceY,
                             "width": width, "height": height, "glyphs": glyphs]
try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
    .write(to: URL(fileURLWithPath: output + ".json"))
for codepoint in [32, 0x2800, 0x28ff] {
    if let g = glyphs.first(where: { $0["codepoint"] as? Int == codepoint }) {
        print(String(format: "U+%04X", codepoint), g["font"]!, "advance:", g["advance"]!)
    }
}
print("Line advance:", advanceY)
