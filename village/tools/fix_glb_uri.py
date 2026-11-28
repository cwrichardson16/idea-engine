"""Point a Kenney GLB's shared texture at art/textures/ (the Exact bake's shared-texture folder)."""
import json, struct, sys

for path in sys.argv[1:]:
    b = open(path, 'rb').read()
    magic, version, _ = struct.unpack('<4sII', b[:12])
    assert magic == b'glTF' and version == 2, path
    n, kind = struct.unpack('<I4s', b[12:20])
    assert kind == b'JSON'
    doc = json.loads(b[20:20 + n])
    rest = b[20 + n:]
    for img in doc.get('images', []):
        if 'uri' in img:
            img['uri'] = 'textures/' + img['uri'].split('/')[-1]
    j = json.dumps(doc, separators=(',', ':')).encode()
    j += b' ' * ((4 - len(j) % 4) % 4)
    out = struct.pack('<I4s', len(j), b'JSON') + j + rest
    open(path, 'wb').write(struct.pack('<4sII', b'glTF', 2, 12 + len(out)) + out)
    print(path, [i.get('uri') for i in doc.get('images', [])])
