"""Writes sheet_columns.json: each Excel sheet's column names, by column index, from icarus's generated sources
(which name them as EXDSchema does). Names only: no game data. `kawari-questdump sheet` labels its columns with them.

    python3 tools/questdump/sheet_columns.py ~/.cargo/git/checkouts/icarus-*/<rev>/src tools/questdump/sheet_columns.json

Run it again when icarus is updated (Cargo.lock's icarus revision.)
"""
import re, json, pathlib, sys
src=pathlib.Path(sys.argv[1])
out={}
pat=re.compile(r'(\w+):\s*row\s*\.columns\[(\d+)\]', re.S)
pat2=re.compile(r'(\w+):\s*\[\s*(row\s*\.columns\[\d+\][^\]]*?,?\s*)+\]', re.S)
for f in sorted(src.glob('*.rs')):
    if f.name=='lib.rs': continue
    t=f.read_text()
    cols={}
    for m in re.finditer(r'(\w+):\s*row\s*\.columns\[(\d+)\]', t):
        cols[int(m.group(2))]=m.group(1)
    # arrays: Name: [ row.columns[a]..., row.columns[b] ...]
    for m in re.finditer(r'(\w+):\s*\[((?:\s*row\s*\.columns\[\d+\][^,\]]*(?:\([^)]*\))*[^,\]]*,?)+)\s*\]', t):
        idx=[int(x) for x in re.findall(r'columns\[(\d+)\]', m.group(2))]
        for i,c in enumerate(idx): cols[c]='%s[%d]'%(m.group(1),i)
    if cols:
        n=max(cols)+1
        out[f.stem]=[cols.get(i,'') for i in range(n)]
json.dump(out, open(sys.argv[2],'w'), separators=(',',':'))
print(len(out))
