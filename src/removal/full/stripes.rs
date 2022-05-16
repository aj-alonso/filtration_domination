use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::Value;

/// A half-open interval.
pub type Interval<VF> = (VF, VF);

/// A vertical or horizontal stripe.
pub type Stripe<VF> = (Interval<VF>, VF);

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum Delimiter<VF> {
    Start(VF, VF),
    End(VF, VF),
}

impl<VF: Value> Ord for Delimiter<VF> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_tuple().cmp(&other.to_tuple())
    }
}

impl<VF: Value> PartialOrd for Delimiter<VF> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<VF: Copy> Delimiter<VF> {
    fn endpoint(&self) -> VF {
        match self {
            Delimiter::Start(e, _) => *e,
            Delimiter::End(e, _) => *e,
        }
    }

    fn to_tuple(self) -> (VF, VF, bool) {
        match self {
            Delimiter::Start(e, v) => (e, v, true),
            Delimiter::End(e, v) => (e, v, false),
        }
    }
}

struct ActiveValues<VF: Value> {
    values: BTreeMap<VF, usize>,
}

impl<VF: Value> ActiveValues<VF> {
    fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    fn add_delimiter(&mut self, delim: Delimiter<VF>) {
        match delim {
            Delimiter::Start(_, v) => {
                let value = self.values.entry(v).or_insert(0);
                *value += 1;
            }
            Delimiter::End(_, v) => {
                if self.values[&v] == 1 {
                    self.values.remove(&v);
                } else {
                    self.values.entry(v).and_modify(|stored_v| *stored_v -= 1);
                }
            }
        }
    }

    fn min(&self) -> Option<VF> {
        self.values.keys().copied().next()
    }
}

#[derive(Debug)]
pub struct Stripes<VF> {
    arranged_stripes: Vec<(VF, VF)>,
}

impl<VF: Value> Stripes<VF> {
    pub fn new(stripes: Vec<Stripe<VF>>) -> Self {
        let mut delimiters = Vec::with_capacity(stripes.len() * 2);
        for s in stripes {
            let ((a, b), v) = s;
            delimiters.push(Delimiter::Start(a, v));
            delimiters.push(Delimiter::End(b, v));
        }

        delimiters.sort_unstable();

        let mut arranged_stripes = Vec::new();
        let mut active_values = ActiveValues::new();

        let mut idx = 0;
        let n = delimiters.len();
        while idx < delimiters.len() {
            let delim = delimiters[idx];
            active_values.add_delimiter(delim);
            idx += 1;

            // Process all delimiters at the same endpoint.
            while idx < n && delimiters[idx].endpoint() == delim.endpoint() {
                active_values.add_delimiter(delimiters[idx]);
                idx += 1;
            }

            let min_value = match active_values.min() {
                None => VF::max_value(),
                Some(v) => v,
            };
            arranged_stripes.push((delim.endpoint(), min_value));
        }

        Self { arranged_stripes }
    }

    pub fn contains_point(&self, p: (VF, VF)) -> bool {
        let pos = self
            .arranged_stripes
            .binary_search_by_key(&p.0, |(e, _)| *e);
        match pos {
            Ok(pos) => self.arranged_stripes[pos].1 <= p.1,
            Err(pos) => {
                if pos == 0 {
                    false
                } else {
                    self.arranged_stripes[pos - 1].1 <= p.1
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.arranged_stripes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::removal::full::stripes::Stripes;

    #[test]
    fn stripes_happy_case() {
        let stripes = Stripes::new(vec![((0, 10), 5)]);

        assert!(stripes.contains_point((5, 5)));
        assert!(stripes.contains_point((1, 5)));
        assert!(stripes.contains_point((0, 5)));
        assert!(stripes.contains_point((3, 50)));

        assert!(!stripes.contains_point((5, 4)));
        assert!(!stripes.contains_point((1, 4)));
        assert!(!stripes.contains_point((0, 4)));
        assert!(!stripes.contains_point((10, 5)));
    }

    #[test]
    fn stripes_start_same_time() {
        let stripes = Stripes::new(vec![((0, 1), 1), ((0, 2), 2), ((0, 3), 3), ((0, 4), 4)]);

        assert!(stripes.contains_point((0, 1)));
        assert!(stripes.contains_point((1, 2)));
        assert!(stripes.contains_point((2, 3)));
        assert!(stripes.contains_point((3, 4)));

        assert!(!stripes.contains_point((1, 1)));
        assert!(!stripes.contains_point((2, 2)));
        assert!(!stripes.contains_point((3, 3)));
        assert!(!stripes.contains_point((4, 4)));
    }

    #[test]
    fn stripes_consecutive() {
        let stripes = Stripes::new(vec![((0, 10), 5), ((10, 20), 4)]);

        assert!(stripes.contains_point((5, 5)));
        assert!(stripes.contains_point((1, 5)));
        assert!(stripes.contains_point((0, 5)));
        assert!(stripes.contains_point((3, 50)));
        assert!(stripes.contains_point((10, 5)));
        assert!(stripes.contains_point((10, 4)));

        assert!(!stripes.contains_point((5, 4)));
        assert!(!stripes.contains_point((1, 4)));
        assert!(!stripes.contains_point((0, 4)));
        assert!(!stripes.contains_point((20, 5)));
    }

    #[test]
    fn stripes_overlap() {
        let stripes = Stripes::new(vec![((0, 10), 5), ((5, 10), 4)]);

        assert!(stripes.contains_point((5, 5)));
        assert!(stripes.contains_point((5, 4)));
        assert!(stripes.contains_point((1, 5)));
        assert!(stripes.contains_point((0, 5)));
        assert!(stripes.contains_point((3, 50)));
        assert!(stripes.contains_point((9, 4)));

        assert!(!stripes.contains_point((1, 4)));
        assert!(!stripes.contains_point((4, 4)));
        assert!(!stripes.contains_point((10, 4)));
    }
}
