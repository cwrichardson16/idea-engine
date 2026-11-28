"""Give Kenney GLBs a cartoon look in the Exact engine: an ink outline
(_OUTLINE_NORMAL/_OUTLINE_REACH + primitive.extras.outline) and a rim light
(material.extras.toon.rim). See ~/Projects/exact/game/render/README.md#toon-models.

Outline normals are averaged across vertices that share a position, so the
hull stays closed on flat-shaded low-poly models. Running it twice is safe.

usage: python3 -I tools/add_outline.py art/*.glb
"""
import json, math, struct, sys

WIDTH = 0.035        # world metres
MIN_PIXELS = 1.4     # CSS pixels, so far-away models still get a line
REACH = 0.12         # local units: the most a vertex may be pushed
INK = [0.02, 0.015, 0.012]  # linear RGB, near-black brown
RIM = 0.35


def read(doc, binary, index):
    a = doc['accessors'][index]
    assert a['componentType'] == 5126 and 'sparse' not in a, 'float accessors only'
    width = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4}[a['type']]
    v = doc['bufferViews'][a['bufferView']]
    stride = v.get('byteStride', 4 * width)
    start = v.get('byteOffset', 0) + a.get('byteOffset', 0)
    return [struct.unpack_from('<%df' % width, binary, start + i * stride) for i in range(a['count'])]


def add(doc, binary, values, kind):
    while len(binary) % 4:
        binary += b'\0'
    width = 3 if kind == 'VEC3' else 1
    data = b''.join(struct.pack('<%df' % width, *v) for v in values)
    doc['bufferViews'].append({'buffer': 0, 'byteOffset': len(binary), 'byteLength': len(data), 'target': 34962})
    doc['accessors'].append({'bufferView': len(doc['bufferViews']) - 1, 'componentType': 5126,
                             'count': len(values), 'type': kind})
    return len(doc['accessors']) - 1, binary + data


for path in sys.argv[1:]:
    b = open(path, 'rb').read()
    magic, version, _ = struct.unpack('<4sII', b[:12])
    assert magic == b'glTF' and version == 2, path
    n, kind = struct.unpack('<I4s', b[12:20])
    assert kind == b'JSON'
    doc = json.loads(b[20:20 + n])
    m, kind2 = struct.unpack('<I4s', b[20 + n:28 + n])
    assert kind2 == b'BIN\0' and len(doc['buffers']) == 1
    binary = bytearray(b[28 + n:28 + n + m])
    changed = False
    for mesh in doc['meshes']:
        for p in mesh['primitives']:
            at = p['attributes']
            if '_OUTLINE_NORMAL' in at:
                continue
            pos = read(doc, binary, at['POSITION'])
            nor = read(doc, binary, at['NORMAL'])
            groups = {}
            for i, q in enumerate(pos):
                key = tuple(round(c, 4) for c in q)
                seen = groups.setdefault(key, {})
                seen[tuple(round(c, 3) for c in nor[i])] = nor[i]
            smooth = {}
            for key, ns in groups.items():
                s = [sum(v[k] for v in ns.values()) for k in range(3)]
                length = math.sqrt(sum(c * c for c in s))
                # Opposite faces at one point (a thin flag) cancel: keep each vertex's own normal.
                smooth[key] = tuple(c / length for c in s) if length > 1e-3 else None

            def unit(v):
                length = math.sqrt(sum(c * c for c in v))
                return tuple(c / length for c in v) if length > 1e-6 else (0.0, 1.0, 0.0)

            outline_normals = [smooth[tuple(round(c, 4) for c in q)] or unit(nor[i]) for i, q in enumerate(pos)]
            i1, binary = add(doc, bytes(binary), outline_normals, 'VEC3')
            i2, binary = add(doc, binary, [(REACH,)] * len(pos), 'SCALAR')
            binary = bytearray(binary)
            at['_OUTLINE_NORMAL'] = i1
            at['_OUTLINE_REACH'] = i2
            p.setdefault('extras', {})['outline'] = {'width': WIDTH, 'minPixels': MIN_PIXELS, 'color': INK}
            changed = True
    for mat in doc.get('materials', []):
        mat.setdefault('extras', {})['toon'] = {'rim': RIM}
    while len(binary) % 4:
        binary += b'\0'
    doc['buffers'][0]['byteLength'] = len(binary)
    j = json.dumps(doc, separators=(',', ':')).encode()
    j += b' ' * ((4 - len(j) % 4) % 4)
    out = struct.pack('<I4s', len(j), b'JSON') + j + struct.pack('<I4s', len(binary), b'BIN\0') + bytes(binary)
    open(path, 'wb').write(struct.pack('<4sII', b'glTF', 2, 12 + len(out)) + out)
    print(path, 'outlined' if changed else 'already outlined')
