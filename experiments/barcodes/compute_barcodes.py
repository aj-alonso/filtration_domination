#!/usr/bin/env python
"""Little utility to compute the barcode of a single-parameter filtered graph using GUDHI."""

import sys
import fileinput
import gudhi

filepath = sys.argv[1]

vertices = set()
edges = []

input_lines = fileinput.input(filepath)
input_lines.readline()

line: str
for line in input_lines:
    string_parts = line.strip().split()
    if len(string_parts) == 3:
        [u, v, v_dist] = line.strip().split()
    else:
        [u, v, v_density, v_dist] = line.strip().split()
    u = int(u)
    v = int(v)
    v_dist = float(v_dist)
    vertices.add(u)
    vertices.add(v)
    edges.append((u, v, v_dist))

simplex = gudhi.simplex_tree.SimplexTree()

for v in vertices:
    simplex.insert([v], 0.0)

for edge in edges:
    simplex.insert([edge[0], edge[1]], edge[2])

simplex.expansion(2)
pers = simplex.persistence(homology_coeff_field=2)
for p in pers:
    print(p[1])
