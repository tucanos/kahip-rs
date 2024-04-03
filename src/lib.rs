//! This crate provides a thin but idiomatic API around KaHIP.

use core::mem;
use core::ptr;
use kahip_sys as m;

pub enum Mode {
    Fast = m::FAST as isize,
    Eco = m::ECO as isize,
    Strong = m::STRONG as isize,
    FastSocial = m::FASTSOCIAL as isize,
    EcoSocial = m::ECOSOCIAL as isize,
    StrongSocial = m::STRONGSOCIAL as isize,
}

pub type Idx = std::os::raw::c_int;

/// Builder structure to setup a graph partition computation.
///
/// This structure holds the required arguments for KaHIP to compute a
/// partition. It also offers methods to easily set any optional argument.
///

#[derive(Debug, PartialEq)]
pub struct Graph<'a> {
    /// The adjency structure of the graph (part 1).
    xadj: &'a mut [Idx],

    /// The adjency structure of the graph (part 2).
    ///
    /// Required size: xadj.last()
    adjncy: &'a mut [Idx],

    /// The computational weights of the vertices.
    ///
    /// Required size: (xadj.len()-1)
    vwgt: Option<&'a mut [Idx]>,

    /// The weight of the edges.
    ///
    /// Required size: xadj.last()
    adjwgt: Option<&'a mut [Idx]>,
}

impl<'a> Graph<'a> {
    /// Creates a new [`Graph`] object to be partitioned.
    ///
    /// # Panics
    ///
    /// This function panics if:
    /// - `xadj` is empty, or
    /// - the length of `adjncy` is different than the last element of `xadj`.
    ///
    /// # Mutability
    ///
    /// While nothing should be modified by the [`Graph`] structure, KaHIP
    /// doesn't specify any `const` modifier, so everything must be mutable on
    /// Rust's side.
    pub fn new(xadj: &'a mut [Idx], adjncy: &'a mut [Idx]) -> Graph<'a> {
        assert_ne!(xadj.len(), 0);
        assert_eq!(adjncy.len(), *xadj.last().unwrap() as usize);

        Graph {
            xadj,
            adjncy,
            adjwgt: None,
            vwgt: None,
        }
    }

    /// Sets the computational weights of the vertices.
    ///
    /// By default all vertices have the same weight.
    pub fn set_vwgt(mut self, vwgt: &'a mut [Idx]) -> Graph<'a> {
        assert_eq!(vwgt.len(), self.xadj.len() - 1);
        self.vwgt = Some(vwgt);
        self
    }

    /// Sets the weights of the edges.
    ///
    /// By default all edges have the same weight.
    pub fn set_adjwgt(mut self, adjwgt: &'a mut [Idx]) -> Graph<'a> {
        assert_eq!(
            adjwgt.len(),
            (*self.xadj.last().unwrap()).try_into().unwrap()
        );
        self.adjwgt = Some(adjwgt);
        self
    }

    /// Partition the graph
    pub fn partition(
        &mut self,
        n_parts: Idx,
        imbalance: f64,
        suppress_output: bool,
        seed: Idx,
        mode: Mode,
    ) -> (Vec<Idx>, Idx) {
        let nvtxs = &mut (self.xadj.len() as Idx - 1) as *mut Idx;
        let xadj = self.xadj.as_mut_ptr();
        let adjncy = self.adjncy.as_mut_ptr();
        let vwgt = if let Some(vwgt) = self.vwgt.as_mut() {
            vwgt.as_mut_ptr()
        } else {
            ptr::null_mut()
        };
        let adjwgt = if let Some(adjwgt) = self.adjwgt.as_mut() {
            adjwgt.as_mut_ptr()
        } else {
            ptr::null_mut()
        };

        let mut edgecut = mem::MaybeUninit::uninit();
        let mut part = vec![0; self.xadj.len() - 1];

        let mut n_parts = n_parts;
        let mut imbalance = imbalance;

        unsafe {
            m::kaffpa(
                nvtxs,
                vwgt,
                xadj,
                adjwgt,
                adjncy,
                &mut n_parts as *mut Idx,
                &mut imbalance as *mut f64,
                suppress_output,
                seed,
                mode as Idx,
                edgecut.as_mut_ptr(),
                part.as_mut_ptr(),
            );
            (part, edgecut.assume_init())
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{Graph, Mode};
    #[test]
    fn test() {
        let mut xadj = vec![0, 2, 5, 7, 9, 12];
        let mut adjncy = vec![1, 4, 0, 2, 4, 1, 3, 2, 4, 0, 1, 3];

        let (part, edgcut) =
            Graph::new(&mut xadj, &mut adjncy).partition(2, 0.03, true, 1234, Mode::Eco);

        assert_eq!(part, [0, 0, 1, 1, 0]);
        assert_eq!(edgcut, 2);
    }
}
