#!/usr/bin/env python3
"""Regenerate the vendored bitmap and CP437 mapping, offline, using stdlib only."""
from pathlib import Path
import re
import hashlib
root=Path(__file__).resolve().parent.parent
source=(root/'assets/moderndos-source.js').read_text()
source=re.sub(r'/\*.*?\*/','',source,flags=re.S)
source=re.sub(r'//[^\n]*','',source)
rows=bytes(int(x,16) for x in re.findall(r'0x([0-9a-fA-F]{2})\b',source))
assert len(rows)==256*16
assert hashlib.sha256(rows).hexdigest()=='78887614b4b94bfa350be2f18aa64b96df52044bbd802cf60e812a9080be1478'
(root/'assets/moderndos-8x16.bin').write_bytes(rows)
low=' ☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼'
chars=low+''.join(chr(i) for i in range(32,127))+'⌂'+''.join(bytes([i]).decode('cp437') for i in range(128,256))
assert len(chars)==256
(root/'assets/cp437.txt').write_text(chars)
print('Font regenerated; checksum verified.')
