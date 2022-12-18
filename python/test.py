#!/usr/bin/env python3
import filtration_domination
import click


@click.command()
@click.argument("filename", type=click.Path(exists=True))
def reduce_edges(filename):
    raw_edges = open(filename, 'r')
    edges = []
    for l in raw_edges.readlines():
        parts = l.split()
        edge = (int(parts[0]), int(parts[1]))
        grade = (float(parts[2]), float(parts[3]))
        edges.append((edge, grade))
    reduced = filtration_domination.remove_strongly_filtration_dominated(edges)
    for edge in reduced:
        print("{} {} {} {}".format(edge[0][0], edge[0][1], edge[1][0], edge[1][1]))


if __name__ == '__main__':
    reduce_edges()
