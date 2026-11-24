#!/usr/bin/env python3
"""Check the staged candidate: native imports, exact code signatures, the flag-rule audit and the
payload inventory. Static analysis only; it proves nothing about gameplay, which needs a user run."""
import ast,json,re,subprocess
from pathlib import Path
from paths import MOD,CONFIG,BUILD_ROOT,CARGO_TARGET,VERSION,input_path
from build import doctor,digest,record,source_hashes
from mhgu_binary import Nro,main_image
from native_support import nso_exports

NRO='lock_on_target.nro'
# Hook targets, mirrored from core/src; the first signature windows.
HOOKS=[0x27e970,0x190d4]
# GE stays untouched: no SEL/MRS/MSR or parallel add/subtract in emitted code.
GE_AFFECTING=re.compile(r'\t(?:sel|mrs|msr|[su]h?(?:add|sub)(?:8|16)|[su]h?(?:asx|sax)|q(?:add|sub)(?:8|16))\b',re.I)
def verify():
    doctor();path=BUILD_ROOT/('package/romfs/nativeNX/plugins/'+NRO)
    data=path.read_bytes();nro=Nro(data)
    elf=CARGO_TARGET/'armv7a-none-eabi/release/liblock_on_target.so'
    # DT_TEXTREL / DF_TEXTREL require writes to memory the Switch maps read-only.
    if 22 in nro.tags or nro.tags.get(30,0)&4:
        raise ValueError('NRO requires dynamic relocations in read-only memory')
    receipt=json.loads((BUILD_ROOT/'build.json').read_text())
    if receipt.get('version')!=VERSION or receipt['nro_sha256']!=digest(path) or receipt.get('source_sha256')!=source_hashes():
        raise ValueError('Build receipt differs from the staged NRO or current sources')
    exports={n.decode() for n,e,*_ in nro.symbols if n and e[5] and e[3]>>4==1}
    if exports!={'forge_onLoad','forge_onInit','forge_onUpdate','init','fini'}:
        raise ValueError('Unexpected plugin exports: '+str(exports))
    imports={n.decode() for n,e,*_ in nro.symbols if n and not e[5] and e[3]>>4==1}
    available=set().union(*(nso_exports(input_path(n)) for n in ['main','sdk','forge']))
    if imports-available: raise ValueError('Unresolved native imports: '+str(imports-available))
    # Every window the plugin checks must match the pinned NSO, or install refuses in game.
    image,_=main_image(input_path('main').read_bytes())
    source=(MOD/'core/src/signatures.rs').read_text();windows=[]
    for offset,raw in re.findall(r'\(0x([0-9a-f]+), \[([^\]]+)\]\)',source):
        expected=bytes(ast.literal_eval('['+raw+']'));offset=int(offset,16)
        if len(expected)!=64 or image[offset:offset+64]!=expected: raise ValueError('Signature mismatch: '+hex(offset))
        windows.append(offset)
    if len(windows)!=6 or windows[:2]!=HOOKS: raise ValueError('Signature table changed')
    disasm=subprocess.check_output([Path(CONFIG['devkitarm'])/'bin/arm-none-eabi-objdump','-d',elf],text=True)
    writes=[line for line in disasm.splitlines() if GE_AFFECTING.search(line)]
    if writes: raise ValueError('GE-affecting instruction emitted: '+str(writes[:5]))
    expected={'romfs/nativeNX/plugins/'+NRO,'manifest.json'}
    package=BUILD_ROOT/'package';actual={str(p.relative_to(package)) for p in package.rglob('*') if p.is_file()}
    if actual!=expected: raise ValueError('Unexpected runtime payload files: '+str(actual))
    if json.loads((package/'manifest.json').read_text())['version']!=f'0.{VERSION[1:]}.0': raise ValueError('manifest.json names another version')
    record('verify.json',{'status':'PASSED','nro_sha256':digest(path),'elf_sha256':digest(elf),
        'exports':sorted(exports),'strong_imports':sorted(imports),'signature_windows':[hex(w) for w in windows],
        'hook_addresses':[hex(h) for h in HOOKS],'hook_abi_bytes':24,'ordinary_hook_slot_bytes':16,'text_relocations':False,
        'ge_affecting_instructions':0,'runtime_test':'NOT RUN'})
    print('PASS: native imports, 6 game windows, GE audit, exports and payload inventory')
if __name__=='__main__': verify()
