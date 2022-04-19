use crate::edges::{BareEdge, FilteredEdge};
use crate::simplicial_complex::{is_sorted, Dimension, SimplicialComplex, Vertex};
use crate::CriticalGrade;
use sorted_iter::assume::AssumeSortedByItemExt;
use sorted_iter::SortedIterator;
use std::collections::BTreeSet;

/// Build a flag multi-filtration from an iterator of multi-filtered edges.
/// The iterator does not need to be sorted.
/// The resulting multi-filtration is 1-critical.
pub fn build_flag_filtration<G: CriticalGrade, S, I: Iterator<Item = FilteredEdge<G>>>(
    vertices: usize,
    max_dim: usize,
    edges: I,
) -> Filtration<G, S>
where
    S: for<'a> SimplicialComplex<'a>,
{
    let mut f: Filtration<G, S> = Filtration::new_empty(vertices, max_dim);
    let mut vertex_simplex = [0];

    // Add vertices.
    for v in 0..vertices {
        vertex_simplex[0] = v;
        f.add(G::zero(), &vertex_simplex);
    }

    let mut neighbours: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); vertices];

    let mut simplex_buffer = BTreeSet::new();
    for filtered_edge in edges {
        let BareEdge(u, v) = filtered_edge.edge;
        simplex_buffer.insert(u);
        simplex_buffer.insert(v);
        f.add_iter(filtered_edge.grade, 1, simplex_buffer.iter().copied());

        let common_neighbours: BTreeSet<usize> = neighbours[u]
            .intersection(&neighbours[v])
            .copied()
            .collect();
        add_flag_simplex(
            &mut f,
            &neighbours,
            max_dim,
            &common_neighbours,
            &mut simplex_buffer,
        );

        neighbours[u].insert(v);
        neighbours[v].insert(u);
        simplex_buffer.clear();
    }

    f
}

fn add_flag_simplex<G: CriticalGrade, S>(
    f: &mut Filtration<G, S>,
    neighbours: &[BTreeSet<usize>],
    max_dim: usize,
    common_neighbours: &BTreeSet<usize>,
    simplex: &mut BTreeSet<usize>,
) where
    S: for<'a> SimplicialComplex<'a>,
{
    // In this call we add simplices of dimension (simplex.len() - 1) + 1.
    let dim = simplex.len();
    if dim > max_dim {
        return;
    }

    for v in common_neighbours.iter() {
        simplex.insert(*v);

        let mut vf = G::min_value();
        for boundary_idx in f
            .simplicial_complex()
            .simplex_boundary(dim, simplex.iter().copied())
        {
            let boundary_vf = f.value_of(dim - 1, boundary_idx);
            vf = vf.join(boundary_vf);
        }

        f.add_iter(vf, dim, simplex.iter().copied());

        if dim < max_dim {
            // Recurse.
            let new_common_neighbours: BTreeSet<usize> = common_neighbours
                .intersection(&neighbours[*v])
                .copied()
                .filter(|x| x < v)
                .collect();
            add_flag_simplex(f, neighbours, max_dim, &new_common_neighbours, simplex);
        }

        simplex.remove(v);
    }
}

#[derive(Debug)]
pub struct Filtration<G, S> {
    /// Critical grade for each cell in each dimension.
    /// First, we index the dimensions, and then the cells.
    grades: Vec<Vec<G>>,

    /// The underlying simplicial complex being filtered.
    complex: S,
}

impl<G: CriticalGrade, S> Filtration<G, S>
where
    S: for<'a> SimplicialComplex<'a>,
{
    pub fn new(complex: S) -> Self {
        let mut grades = Vec::with_capacity(complex.max_dimension() + 1);
        for dim in 0..complex.max_dimension() + 1 {
            grades.push(vec![G::min_value(); complex.n_cells(dim)]);
        }
        Filtration { grades, complex }
    }

    pub fn new_empty(max_vertices: Vertex, max_dim: Dimension) -> Self {
        let s = S::new(max_vertices, max_dim);
        Self::new(s)
    }

    pub fn add(&mut self, g: G, s: &[Vertex]) -> Option<(Dimension, usize)> {
        assert!(is_sorted(s), "To add a simplex it must be sorted first.");

        let dim = s.len() - 1;
        self.add_iter(g, dim, s.iter().copied().assume_sorted_by_item())
    }

    pub fn add_iter<I: SortedIterator<Item = usize>>(
        &mut self,
        g: G,
        dim: Dimension,
        iter: I,
    ) -> Option<(Dimension, usize)> {
        let added_simplex = self.complex.add_iter(dim, iter);
        if let Some((dimension, idx)) = added_simplex {
            assert_eq!(idx, self.grades[dimension].len(),
                       "Programming error: the index of an added simplex is the total number of simplices in that dimension.");
            if dimension > 0 {
                for boundary_idx in self.complex.boundary_iterator(dimension, idx) {
                    // TODO: revisit this.
                    assert!(self.grades[dimension - 1][boundary_idx].lte(&g), "The grade of a simplex is greater than or equal to the grade of its facets: {:?} is not lte than {:?}.", self.grades[dimension - 1][boundary_idx], g);
                }
            }
            self.grades[dimension].push(g);
        }
        added_simplex
    }

    pub fn value_of(&self, dim: Dimension, idx: usize) -> &G {
        &self.grades[dim][idx]
    }

    pub fn simplicial_complex(&self) -> &S {
        &self.complex
    }
}
