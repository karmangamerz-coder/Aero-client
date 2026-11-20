# SPDX-License-Identifier: MIT
"""Deterministic original Aero vector artwork. Python standard library only."""
from pathlib import Path
import math

root = Path(__file__).resolve().parents[1] / 'native/launcher/assets'
root.mkdir(parents=True, exist_ok=True)
(root / 'landscape.svg').write_text('''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 800">
<defs><linearGradient id="sky" x2="0" y2="1"><stop stop-color="#70C8F0"/><stop offset="1" stop-color="#E4F8FF"/></linearGradient>
<linearGradient id="grass" x2="0" y2="1"><stop stop-color="#99D94B"/><stop offset="1" stop-color="#369957"/></linearGradient>
<linearGradient id="water" x2="0" y2="1"><stop stop-color="#C4F3F4"/><stop offset="1" stop-color="#51AFD3"/></linearGradient>
<radialGradient id="sun"><stop stop-color="#FFFCE5" stop-opacity=".9"/><stop offset="1" stop-color="#FFFCE5" stop-opacity="0"/></radialGradient></defs>
<path fill="url(#sky)" d="M0 0h1200v800H0z"/><circle cx="780" cy="120" r="300" fill="url(#sun)"/>
<g fill="white" opacity=".72"><ellipse cx="205" cy="166" rx="180" ry="30"/><ellipse cx="170" cy="145" rx="73" ry="40"/><ellipse cx="260" cy="157" rx="65" ry="32"/><ellipse cx="920" cy="265" rx="194" ry="24"/><ellipse cx="970" cy="246" rx="85" ry="37"/></g>
<path fill="#A9DCA7" d="M0 490Q210 337 450 480T900 432T1200 457V800H0z"/>
<path fill="url(#grass)" d="M0 540Q130 426 330 515T780 545T1200 430V800H0z"/>
<path fill="url(#water)" d="M0 665Q290 564 610 652T1200 610V800H0z"/>
<path fill="#EDFDF8" opacity=".55" d="M0 668Q290 568 610 655T1200 613v5Q890 710 610 660T0 674z"/>
<g stroke="white" stroke-width="2" opacity=".35"><path d="M90 717h185m-80 24h245m300-24h155m-75 36h220"/></g>
<path fill="#4DAB51" d="M0 766Q170 694 350 772L410 800H0z"/>
<g fill="white" fill-opacity=".12" stroke="white" stroke-opacity=".4"><circle cx="87" cy="390" r="22"/><circle cx="1080" cy="130" r="16"/></g></svg>''', encoding='utf-8')
(root / 'logo.svg').write_text('''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 48 48"><defs><linearGradient id="a" x2="0" y2="1"><stop stop-color="#4DD2EB"/><stop offset="1" stop-color="#087EBA"/></linearGradient></defs><circle cx="24" cy="24" r="21" fill="url(#a)" stroke="#D4F9FF"/><path d="m11 32 12-21h3l12 21h-7l-6-12-7 12z" fill="white"/><path d="M7 30q17-12 34-4" fill="none" stroke="#9DE967" stroke-width="4"/></svg>''', encoding='utf-8')

# Orthographic cuboid projection: no engine, textures, borrowed skins or runtime animation.
for frame, angle in enumerate(range(-48, 49, 12)):
    a = math.radians(angle)
    faces = []
    def project(p):
        x, y, z = p
        rx = x * math.cos(a) + z * math.sin(a)
        rz = z * math.cos(a) - x * math.sin(a)
        return 135 + rx * 7.4, 304 - y * 7.4 - rz * 2.0, rz
    def face(points, color):
        projected = [project(p) for p in points]
        depth = sum(p[2] for p in projected) / len(projected)
        coords = ' '.join(f'{x:.2f},{y:.2f}' for x,y,_ in projected)
        faces.append((depth, f'<polygon points="{coords}" fill="{color}"/>'))
    def box(x, y, z, w, h, d, front, side, top):
        face([(x,y,z+d),(x+w,y,z+d),(x+w,y+h,z+d),(x,y+h,z+d)],front)
        face([(x,y,z),(x+w,y,z),(x+w,y+h,z),(x,y+h,z)],side)
        face([(x,y,z),(x,y,z+d),(x,y+h,z+d),(x,y+h,z)],side)
        face([(x+w,y,z),(x+w,y,z+d),(x+w,y+h,z+d),(x+w,y+h,z)],side)
        face([(x,y+h,z),(x+w,y+h,z),(x+w,y+h,z+d),(x,y+h,z+d)],top)
    box(-4,1,-2,3.8,12,4,'#265670','#193D59','#417D91')
    box(.2,1,-2,3.8,12,4,'#30708B','#24506E','#5693A0')
    box(-4,0,-2.2,3.8,2,4.5,'#173B50','#112E44','#356275')
    box(.2,0,-2.2,3.8,2,4.5,'#173B50','#112E44','#356275')
    box(-4,13,-2,8,12,4,'#25B8C7','#147E9D','#89E4DF')
    box(-8,14,-2,4,10,4,'#3DC8CD','#21879B','#A4EBE1')
    box(4,14,-2,4,10,4,'#3DC8CD','#21879B','#A4EBE1')
    box(-8,12,-2,4,4,4,'#E1B28A','#BA876C','#F5C9A0')
    box(4,12,-2,4,4,4,'#E1B28A','#BA876C','#F5C9A0')
    box(-4,25,-4,8,8,8,'#E1B28A','#BB8D70','#ECC39B')
    box(-4.1,31,-4.1,8.2,2.2,8.2,'#594532','#3D352D','#786043')
    # Face and jacket insignia lie just in front of their cuboids.
    for x in [-2.6,1.3]:
        face([(x,28.8,4.02),(x+1.3,28.8,4.02),(x+1.3,29.8,4.02),(x,29.8,4.02)], '#F1FFFF')
        face([(x+.4,28.8,4.04),(x+1,28.8,4.04),(x+1,29.8,4.04),(x+.4,29.8,4.04)], '#20647A')
    face([(-1,26.5,4.05),(1.2,26.5,4.05),(1.2,27,4.05),(-1,27,4.05)], '#A5755C')
    face([(-1.8,18,2.1),(0,22,2.1),(1.8,18,2.1),(0,19,2.1)], '#E7FFFF')
    svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 270 342"><ellipse cx="135" cy="318" rx="76" ry="14" fill="#246C79" opacity=".18"/>'
    svg += ''.join(fragment for _,fragment in sorted(faces,key=lambda f:f[0])) + '</svg>'
    (root / f'character-{frame}.svg').write_text(svg,encoding='utf-8')
print('Generated 11 original SVG assets.')
