# Filtration-domination algorithms

The present code implements algorithms to remove edges from a bifiltered graph,
while maintaining the topological properties of its clique complex. All these
terms, and a description of the algorithms, is in the paper

"Filtration-Domination in Bifiltered Graphs".

## Usage

To use the algorithms, first add the present Rust crate as a dependency to your
Rust project.

The API has two main functions:
``filtration_domination::removal::remove_filtration_dominated`` and
``filtration_domination::removal::remove_strongly_filtration_dominated``, which
directly correspond to the described algorithms in the paper. They take as input a list of (bifiltered) edges, encoded as an ``EdgeList``, and output a reduced list of edges.
