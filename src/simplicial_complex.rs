use rustc_hash::FxHashMap;
use sorted_iter::assume::AssumeSortedByItemExt;
use sorted_iter::SortedIterator;
use std::collections::hash_map::Entry;

pub type Vertex = usize;
pub type Dimension = usize;

pub trait SimplicialComplex<'a> {
    type BoundaryIterator: Iterator<Item = usize> + 'a;

    type VertexIterator: Iterator<Item = Vertex>;

    fn new(max_vertices: Vertex, max_dim: Dimension) -> Self;

    fn max_dimension(&self) -> Dimension;

    fn n_cells(&self, dim: Dimension) -> usize;

    /// Add a simplex, given by a vector of vertices, to the simplicial complex.
    /// The vector of vertices must be ordered, and the facets of the simplex must be already in the simplicial complex.
    /// Returns an option, with no value if the simplex is already in the simplex,
    /// and with the dimension and the index of its basis otherwise.
    fn add(&mut self, s: &[Vertex]) -> Option<(Dimension, usize)>;

    /// Add a simplex, given by a iterator that produces vertices, to the simplicial complex.
    /// The iterator must produce ordered items.
    /// The iterator must produce exactly dim + 1 items.
    /// The boundaries of the simplex must have been added before.
    fn add_iter<I: SortedIterator<Item = usize>>(
        &mut self,
        dim: Dimension,
        iter: I,
    ) -> Option<(Dimension, usize)>;

    /// Returns an iterator over the boundary of the simplex of the given index.
    /// The iterator returns the indexes of the simplexes in the boundary.
    fn boundary_iterator(&'a self, dim: Dimension, idx: usize) -> Self::BoundaryIterator;

    /// Returns an iterator over the boundary of the given simplex.
    /// Unlike `boundary_iterator`, the simplex may not be in the simplicial complex,
    /// but it faces must be.
    fn simplex_boundary<I: SortedIterator<Item = usize>>(
        &'a self,
        dim: Dimension,
        simplex_iter: I,
    ) -> Self::BoundaryIterator;

    /// Returns an iterator over the vertices of the simplex of the given index.
    fn simplex_vertices(&self, dim: Dimension, idx: usize) -> Self::VertexIterator;
}

/// A SimplexKey encodes a simplex as a non-negative integer.
type SimplexKey = usize;

#[derive(Default, Debug)]
pub struct MapSimplicialComplex {
    /// Associates a simplex id to its key.
    /// The ith-element of the vector contains the simplices of the dimension i.
    simplices_by_dim: Vec<Vec<SimplexKey>>,

    /// Associates a simplex key to its index in the vector of its dimension in simplices_by_dim.
    key_to_idx: Vec<FxHashMap<SimplexKey, usize>>,

    /// Maximum number of vertices.
    max_n: Vertex,
}

impl MapSimplicialComplex {
    pub fn new(max_vertices: Vertex, max_dim: Dimension) -> Self {
        let mut s = MapSimplicialComplex {
            max_n: max_vertices,
            ..Default::default()
        };
        s.simplices_by_dim.resize(max_dim + 1, Default::default());
        s.key_to_idx.resize(max_dim + 1, Default::default());
        s
    }

    /// Get the simplex key from a stream of vertices.
    fn simplex_to_key<I: SortedIterator<Item = usize>>(&self, iter: I) -> SimplexKey {
        let mut k: SimplexKey = 0;
        let mut exp: SimplexKey = 1;
        for v in iter {
            k += v * exp;
            exp *= self.max_n;
        }
        k
    }

    fn add_simplex_key_check_boundaries(
        &mut self,
        dim: Dimension,
        key: SimplexKey,
    ) -> Option<(Dimension, usize)> {
        if dim > 0 {
            let it = SimplexKeyBoundaryIterator::new(self.max_n, dim, key);

            for facet_key in it {
                assert!(
                    self.has_simplex_key(dim - 1, &facet_key),
                    "Adding a simplex requires that its boundaries have been added before."
                )
            }
        }
        self.add_simplex_key(dim, key)
    }

    fn add_simplex_key(&mut self, dim: Dimension, key: SimplexKey) -> Option<(Dimension, usize)> {
        match self.key_to_idx[dim].entry(key) {
            Entry::Occupied(_) => None,
            Entry::Vacant(entry) => {
                if dim == 0 {
                    assert!(
                        self.simplices_by_dim[0].len() < self.max_n,
                        "Exceeded the maximum number of vertices."
                    );
                }
                let idx = self.simplices_by_dim[dim].len();

                self.simplices_by_dim[dim].push(key);
                entry.insert(idx);

                Some((dim, idx))
            }
        }
    }

    fn has_simplex_key(&self, dim: Dimension, k: &SimplexKey) -> bool {
        self.key_to_idx[dim].contains_key(k)
    }
}

impl<'a> SimplicialComplex<'a> for MapSimplicialComplex {
    type BoundaryIterator = MapBoundaryIterator<'a>;
    type VertexIterator = SimplexKeyVertexIterator;

    fn new(max_n: Vertex, max_dim: Dimension) -> Self {
        Self::new(max_n, max_dim)
    }

    fn max_dimension(&self) -> Dimension {
        self.simplices_by_dim.len() - 1
    }

    fn n_cells(&self, dim: Dimension) -> usize {
        self.simplices_by_dim[dim].len()
    }

    fn add(&mut self, s: &[Vertex]) -> Option<(Dimension, usize)> {
        assert!(is_sorted(s), "To add a simplex it must be sorted first.");

        let dim = s.len() - 1;
        let k = self.simplex_to_key(s.iter().copied().assume_sorted_by_item());

        self.add_simplex_key_check_boundaries(dim, k)
    }

    fn add_iter<I: SortedIterator<Item = usize>>(
        &mut self,
        dim: Dimension,
        iter: I,
    ) -> Option<(Dimension, usize)> {
        let key = self.simplex_to_key(iter);
        self.add_simplex_key_check_boundaries(dim, key)
    }

    fn boundary_iterator(&'a self, dim: Dimension, idx: usize) -> Self::BoundaryIterator {
        MapBoundaryIterator::new(self, dim, self.simplices_by_dim[dim][idx])
    }

    fn simplex_boundary<I: SortedIterator<Item = usize>>(
        &'a self,
        dim: Dimension,
        simplex_iter: I,
    ) -> Self::BoundaryIterator {
        MapBoundaryIterator::new(self, dim, self.simplex_to_key(simplex_iter))
    }

    fn simplex_vertices(&self, dim: Dimension, idx: usize) -> Self::VertexIterator {
        SimplexKeyVertexIterator::new(dim, self.simplices_by_dim[dim][idx], self.max_n)
    }
}

pub struct MapBoundaryIterator<'a> {
    complex: &'a MapSimplicialComplex,

    simplex_key_iterator: SimplexKeyBoundaryIterator,
}

impl MapBoundaryIterator<'_> {
    fn new(
        complex: &'_ MapSimplicialComplex,
        dimension: Dimension,
        key: SimplexKey,
    ) -> MapBoundaryIterator<'_> {
        MapBoundaryIterator {
            complex,
            simplex_key_iterator: SimplexKeyBoundaryIterator::new(complex.max_n, dimension, key),
        }
    }
}

impl Iterator for MapBoundaryIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.simplex_key_iterator.dimension == 0 {
            return None;
        }

        let next_key = self.simplex_key_iterator.next();
        if let Some(key) = next_key {
            let facet_dimension = self.simplex_key_iterator.dimension - 1;
            Some(self.complex.key_to_idx[facet_dimension][&key])
        } else {
            None
        }
    }
}

struct SimplexKeyBoundaryIterator {
    dimension: Dimension,
    max_n: Vertex,
    iteration: Dimension,
    current_power: Vertex,
    left_to_process: SimplexKey,
    processed: SimplexKey,
}

impl SimplexKeyBoundaryIterator {
    fn new(max_n: Vertex, dimension: Dimension, key: SimplexKey) -> SimplexKeyBoundaryIterator {
        SimplexKeyBoundaryIterator {
            max_n,
            dimension,
            iteration: 0,
            left_to_process: key,
            processed: 0,
            current_power: 1,
        }
    }
}

impl Iterator for SimplexKeyBoundaryIterator {
    type Item = SimplexKey;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iteration == self.dimension + 1 {
            return None;
        }
        let next_power = self.current_power * self.max_n;
        let removed_v = self.left_to_process % self.max_n;

        self.left_to_process /= self.max_n;
        let face = self.left_to_process * self.current_power + self.processed;

        self.processed += removed_v * self.current_power;

        self.current_power = next_power;
        self.iteration += 1;
        Some(face)
    }
}

pub struct SimplexKeyVertexIterator {
    key: usize,
    vertices_left: usize,
    modulo: usize,
}

impl SimplexKeyVertexIterator {
    fn new(dim: usize, key: usize, modulo: usize) -> SimplexKeyVertexIterator {
        SimplexKeyVertexIterator {
            key,
            vertices_left: dim + 1,
            modulo,
        }
    }
}

impl Iterator for SimplexKeyVertexIterator {
    type Item = Vertex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.vertices_left == 0 {
            return None;
        }
        let v = self.key % self.modulo;
        self.key /= self.modulo;
        self.vertices_left -= 1;
        Some(v)
    }
}

pub(crate) fn is_sorted<T: Ord>(data: &[T]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}

#[cfg(test)]
mod tests {
    use crate::simplicial_complex::MapSimplicialComplex;
    use crate::simplicial_complex::SimplicialComplex;

    #[test]
    fn simplex_add_one_by_one() {
        let mut s = MapSimplicialComplex::new(10, 10);
        s.add(&[0usize]);
        s.add(&[1usize]);
        s.add(&[2usize]);
        s.add(&[0usize, 1usize]);
        s.add(&[1usize, 2usize]);
        s.add(&[0usize, 2usize]);
        s.add(&[0usize, 1usize, 2usize]);
        // No errors should have been raised.
    }

    #[test]
    fn boundary_iterator_happy_case() {
        let mut s = MapSimplicialComplex::new(10, 10);
        s.add(&[0usize]);
        s.add(&[1usize]);
        s.add(&[2usize]);
        s.add(&[0usize, 1usize]);
        s.add(&[1usize, 2usize]);
        s.add(&[0usize, 2usize]);
        let (dim, idx) = s.add(&[0usize, 1usize, 2usize]).unwrap();
        let it = s.boundary_iterator(dim, idx);
        let result: Vec<_> = it.collect();
        assert_eq!(vec![1, 2, 0], result);
    }

    #[test]
    fn vertices_iterator_happy_case() {
        let mut s = MapSimplicialComplex::new(10, 10);
        s.add(&[0usize]);
        s.add(&[1usize]);
        s.add(&[2usize]);
        s.add(&[0usize, 1usize]);
        s.add(&[1usize, 2usize]);
        s.add(&[0usize, 2usize]);
        let (dim, idx) = s.add(&[0usize, 1usize, 2usize]).unwrap();
        let vertices: Vec<usize> = s.simplex_vertices(dim, idx).collect();
        assert_eq!(vertices, [0, 1, 2]);
    }
}
