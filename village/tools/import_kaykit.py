"""Import a KayKit model (CC0, art/LICENSE-kaykit-*.txt) into art/ as a GLB the
Exact bake accepts, keeping only what the game draws.

  building  SRC.gltf OUT.glb
      A Medieval Hexagon building (.gltf + .bin beside it). The atlas
      (hexagons_medieval.png) is shared: the image uri points at
      art/textures/, so every building downloads it once.

  character SRC.glb OUT.glb --clips A,B,.. --meshes N1,N2,.. --texture art/textures/x.png
      An Adventurers character. Keeps only the named clips and mesh nodes
      (every other held item is dropped), moves the embedded texture to
      art/textures/, and repacks the buffer so dropped data is gone.

Then run tools/add_outline.py on OUT.glb. Pure Python, no dependencies; it
only reads the source files, never runs anything from them.

usage: python3 -I tools/import_kaykit.py building|character SRC OUT [options]
"""
import json, os, struct, sys


def read_glb(path):
    b = open(path, 'rb').read()
    magic, version, _ = struct.unpack('<4sII', b[:12])
    assert magic == b'glTF' and version == 2, path
    n, kind = struct.unpack('<I4s', b[12:20])
    assert kind == b'JSON', path
    doc = json.loads(b[20:20 + n])
    m, kind2 = struct.unpack('<I4s', b[20 + n:28 + n])
    assert kind2 == b'BIN\0' and len(doc['buffers']) == 1, path
    return doc, b[28 + n:28 + n + m]


def read_gltf(path):
    doc = json.load(open(path))
    assert len(doc['buffers']) == 1, path
    uri = doc['buffers'][0]['uri']
    assert '/' not in uri and '..' not in uri and not uri.startswith('data:'), uri
    return doc, open(os.path.join(os.path.dirname(path), uri), 'rb').read()


def write_glb(path, doc, binary):
    binary = bytes(binary) + b'\0' * ((4 - len(binary) % 4) % 4)
    doc['buffers'] = [{'byteLength': len(binary)}]
    j = json.dumps(doc, separators=(',', ':')).encode()
    j += b' ' * ((4 - len(j) % 4) % 4)
    out = struct.pack('<I4s', len(j), b'JSON') + j + struct.pack('<I4s', len(binary), b'BIN\0') + binary
    open(path, 'wb').write(struct.pack('<4sII', b'glTF', 2, 12 + len(out)) + out)


def repack(doc, binary):
    """Drop meshes no node uses, then every accessor, view, image, texture and
    material nothing kept reaches, and rebuild one tight buffer."""
    used_meshes = sorted({n['mesh'] for n in doc['nodes'] if 'mesh' in n})
    mesh_map = {old: new for new, old in enumerate(used_meshes)}
    doc['meshes'] = [doc['meshes'][i] for i in used_meshes]
    for n in doc['nodes']:
        if 'mesh' in n:
            n['mesh'] = mesh_map[n['mesh']]
    acc = set()
    mats = set()
    for m in doc['meshes']:
        for p in m['primitives']:
            assert 'targets' not in p, 'morph targets are refused by the bake'
            acc.update(p['attributes'].values())
            if 'indices' in p:
                acc.add(p['indices'])
            if 'material' in p:
                mats.add(p['material'])
    for s in doc.get('skins', []):
        if 'inverseBindMatrices' in s:
            acc.add(s['inverseBindMatrices'])
    for a in doc.get('animations', []):
        for s in a['samplers']:
            acc.update((s['input'], s['output']))
    acc = sorted(acc)
    acc_map = {old: new for new, old in enumerate(acc)}
    # Materials -> textures -> images.
    mats = sorted(mats)
    mat_map = {old: new for new, old in enumerate(mats)}
    doc['materials'] = [doc['materials'][i] for i in mats]
    tex = set()
    for m in doc['materials']:
        for slot in (m.get('pbrMetallicRoughness', {}), m):
            for key, val in slot.items():
                if key.endswith('Texture') and isinstance(val, dict):
                    tex.add(val['index'])
    tex = sorted(tex)
    tex_map = {old: new for new, old in enumerate(tex)}
    doc['textures'] = [doc['textures'][i] for i in tex] if tex else []
    for m in doc['materials']:
        for slot in (m.get('pbrMetallicRoughness', {}), m):
            for key, val in slot.items():
                if key.endswith('Texture') and isinstance(val, dict):
                    val['index'] = tex_map[val['index']]
    imgs = sorted({t['source'] for t in doc['textures']})
    img_map = {old: new for new, old in enumerate(imgs)}
    doc['images'] = [doc['images'][i] for i in imgs]
    for t in doc['textures']:
        t['source'] = img_map[t['source']]
    for m in doc['meshes']:
        for p in m['primitives']:
            p['attributes'] = {k: acc_map[v] for k, v in p['attributes'].items()}
            if 'indices' in p:
                p['indices'] = acc_map[p['indices']]
            if 'material' in p:
                p['material'] = mat_map[p['material']]
    for s in doc.get('skins', []):
        if 'inverseBindMatrices' in s:
            s['inverseBindMatrices'] = acc_map[s['inverseBindMatrices']]
    for a in doc.get('animations', []):
        for s in a['samplers']:
            s['input'], s['output'] = acc_map[s['input']], acc_map[s['output']]
    # Views: one per surviving use, copied in order into a fresh buffer.
    old_views = doc['bufferViews']
    views, view_map, out = [], {}, bytearray()

    def keep_view(i):
        if i not in view_map:
            v = dict(old_views[i])
            start = v.get('byteOffset', 0)
            data = binary[start:start + v['byteLength']]
            while len(out) % 4:
                out.append(0)
            v['buffer'], v['byteOffset'] = 0, len(out)
            out.extend(data)
            view_map[i] = len(views)
            views.append(v)
        return view_map[i]

    accessors = []
    for i in acc:
        a = dict(doc['accessors'][i])
        assert 'sparse' not in a, 'sparse accessors are refused by the bake'
        a['bufferView'] = keep_view(a['bufferView'])
        accessors.append(a)
    for img in doc['images']:
        if 'bufferView' in img:
            img['bufferView'] = keep_view(img['bufferView'])
    doc['accessors'], doc['bufferViews'] = accessors, views
    for key in ('textures', 'images', 'samplers'):
        if key in doc and not doc[key]:
            del doc[key]
    return doc, out


def building(src, dst):
    doc, binary = read_gltf(src)
    for img in doc.get('images', []):
        img['uri'] = 'textures/' + img['uri'].split('/')[-1]
    doc, binary = repack(doc, binary)
    write_glb(dst, doc, binary)


def character(src, dst, clips, meshes, texture):
    doc, binary = read_glb(src)
    names = {a['name'] for a in doc['animations']}
    missing = [c for c in clips if c not in names]
    assert not missing, 'no such clips: %s' % missing
    doc['animations'] = [a for a in doc['animations'] if a['name'] in clips]
    have = {n.get('name') for n in doc['nodes'] if 'mesh' in n}
    missing = [m for m in meshes if m not in have]
    assert not missing, 'no such mesh nodes: %s' % missing
    for n in doc['nodes']:
        if 'mesh' in n and n.get('name') not in meshes:
            del n['mesh']
    doc, binary = repack(doc, binary)
    # The embedded texture moves to art/textures/ (shared, baked once).
    assert len(doc.get('images', [])) == 1, 'one texture expected'
    img = doc['images'][0]
    v = doc['bufferViews'][img.pop('bufferView')]
    png = bytes(binary[v['byteOffset']:v['byteOffset'] + v['byteLength']])
    assert png[:8] == b'\x89PNG\r\n\x1a\n', 'PNG expected'
    open(texture, 'wb').write(png)
    img.pop('mimeType', None)
    img['uri'] = 'textures/' + os.path.basename(texture)
    doc, binary = repack(doc, binary)  # drop the now-unused image view
    write_glb(dst, doc, binary)


def opt(name):
    args = sys.argv
    return args[args.index(name) + 1] if name in args else None


if __name__ == '__main__':
    mode, src, dst = sys.argv[1:4]
    if mode == 'building':
        building(src, dst)
    elif mode == 'character':
        character(src, dst, opt('--clips').split(','), opt('--meshes').split(','), opt('--texture'))
    else:
        sys.exit(__doc__)
    print(dst, os.path.getsize(dst), 'bytes')
