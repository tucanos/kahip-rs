#![warn(clippy::all)]

//! This crate provides a thin but idiomatic API around KaHIP.

use core::fmt::Debug;
use core::mem;
use core::ptr;
use kahip_sys as m;

#[derive(Clone, Copy, Debug)]
pub enum KahipMode {
    Fast = m::FAST as isize,
    Eco = m::ECO as isize,
    Strong = m::STRONG as isize,
    FastSocial = m::FASTSOCIAL as isize,
    EcoSocial = m::ECOSOCIAL as isize,
    StrongSocial = m::STRONGSOCIAL as isize,
}

#[derive(Clone, Copy, Debug)]
pub struct KahipParams {
    pub imbalance: f64,
    pub suppress_output: bool,
    pub seed: Idx,
    pub mode: KahipMode,
}

impl Default for KahipParams {
    fn default() -> Self {
        Self {
            imbalance: 0.03,
            suppress_output: true,
            seed: 0,
            mode: KahipMode::Eco,
        }
    }
}

pub type Idx = m::kahip_idx;
pub type KaminparNodeId = m::kaminpar_node_id_t;
pub type KaminparEdgeId = m::kaminpar_edge_id_t;
pub type KaminparNodeWeight = m::kaminpar_node_weight_t;
pub type KaminparEdgeWeight = m::kaminpar_edge_weight_t;
pub type KaminparBlockId = m::kaminpar_block_id_t;

/// Builder structure to setup a graph partition computation.
///
/// This structure holds the required arguments for KaHIP to compute a
/// partition. It also offers methods to easily set any optional argument.
///

#[derive(Debug, PartialEq)]
pub struct CsrGraph<'a, O, N, NW, EW> {
    /// The adjency structure of the graph (part 1).
    xadj: &'a mut [O],

    /// The adjency structure of the graph (part 2).
    ///
    /// Required size: xadj.last()
    adjncy: &'a mut [N],

    /// The computational weights of the vertices.
    ///
    /// Required size: (xadj.len()-1)
    vwgt: Option<&'a mut [NW]>,

    /// The weight of the edges.
    ///
    /// Required size: xadj.last()
    adjwgt: Option<&'a mut [EW]>,
}

impl<'a, O, N, NW, EW> CsrGraph<'a, O, N, NW, EW>
where
    O: Copy + TryInto<usize>,
    <O as TryInto<usize>>::Error: Debug,
{
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
    pub fn new(xadj: &'a mut [O], adjncy: &'a mut [N]) -> CsrGraph<'a, O, N, NW, EW> {
        assert_ne!(xadj.len(), 0);
        assert_eq!(adjncy.len(), (*xadj.last().unwrap()).try_into().unwrap());

        CsrGraph {
            xadj,
            adjncy,
            adjwgt: None,
            vwgt: None,
        }
    }

    /// Sets the computational weights of the vertices.
    ///
    /// By default all vertices have the same weight.
    pub fn set_vwgt(mut self, vwgt: &'a mut [NW]) -> CsrGraph<'a, O, N, NW, EW> {
        assert_eq!(vwgt.len(), self.xadj.len() - 1);
        self.vwgt = Some(vwgt);
        self
    }

    /// Sets the weights of the edges.
    ///
    /// By default all edges have the same weight.
    pub fn set_adjwgt(mut self, adjwgt: &'a mut [EW]) -> CsrGraph<'a, O, N, NW, EW> {
        assert_eq!(
            adjwgt.len(),
            (*self.xadj.last().unwrap()).try_into().unwrap()
        );
        self.adjwgt = Some(adjwgt);
        self
    }
}

pub type KaHIPGraph<'a> = CsrGraph<'a, Idx, Idx, Idx, Idx>;
pub type KaMinParGraph<'a> =
    CsrGraph<'a, KaminparEdgeId, KaminparNodeId, KaminparNodeWeight, KaminparEdgeWeight>;

impl<'a> KaHIPGraph<'a> {
    /// Partition the graph
    pub fn partition(&mut self, n_parts: Idx, params: KahipParams) -> (Vec<Idx>, Idx) {
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
        let mut imbalance = params.imbalance;

        unsafe {
            m::kaffpa(
                nvtxs,
                vwgt,
                xadj,
                adjwgt,
                adjncy,
                &mut n_parts as *mut Idx,
                &mut imbalance as *mut f64,
                params.suppress_output,
                params.seed,
                params.mode as Idx,
                edgecut.as_mut_ptr(),
                part.as_mut_ptr(),
            );
            (part, edgecut.assume_init())
        }
    }

    /// Compute a node ordering using KaHIP reduced nested dissection.
    pub fn reduced_nd_ordering(
        &mut self,
        suppress_output: bool,
        seed: Idx,
        mode: KahipMode,
    ) -> Vec<Idx> {
        let mut n = self.xadj.len() as Idx - 1;
        let mut ordering = vec![0; self.xadj.len() - 1];

        unsafe {
            m::reduced_nd(
                &mut n as *mut Idx,
                self.xadj.as_mut_ptr(),
                self.adjncy.as_mut_ptr(),
                suppress_output,
                seed,
                mode as Idx,
                ordering.as_mut_ptr(),
            );
        }

        ordering
    }
}

#[derive(Clone, Copy, Debug)]
pub enum KaminparPreset {
    Default,
    Strong,
    TeraPart,
    LargeK,
    VCycle,
}

#[derive(Clone, Copy, Debug)]
pub enum KaminparOutputLevel {
    Quiet,
    Progress,
    Application,
    Experiment,
    Debug,
}

impl KaminparOutputLevel {
    fn as_raw(&self) -> m::kaminpar_output_level_t {
        match self {
            Self::Quiet => m::kaminpar_output_level_t_KAMINPAR_OUTPUT_LEVEL_QUIET,
            Self::Progress => m::kaminpar_output_level_t_KAMINPAR_OUTPUT_LEVEL_PROGRESS,
            Self::Application => m::kaminpar_output_level_t_KAMINPAR_OUTPUT_LEVEL_APPLICATION,
            Self::Experiment => m::kaminpar_output_level_t_KAMINPAR_OUTPUT_LEVEL_EXPERIMENT,
            Self::Debug => m::kaminpar_output_level_t_KAMINPAR_OUTPUT_LEVEL_DEBUG,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KaminparParams {
    pub epsilon: f64,
    pub num_threads: i32,
    pub preset: KaminparPreset,
    pub output_level: KaminparOutputLevel,
}

impl Default for KaminparParams {
    fn default() -> Self {
        let num_threads = match std::thread::available_parallelism() {
            Ok(n) => i32::try_from(n.get()).unwrap_or(i32::MAX),
            Err(_) => 1,
        };

        Self {
            epsilon: 0.03,
            num_threads,
            preset: KaminparPreset::Default,
            output_level: KaminparOutputLevel::Application,
        }
    }
}

pub struct KaMinParPartitioner {
    ctx: *mut m::kaminpar_context_t,
    ptr: *mut m::kaminpar_t,
}

impl KaMinParPartitioner {
    pub fn new(num_threads: i32, preset: KaminparPreset) -> KaMinParPartitioner {
        let ctx = unsafe {
            match preset {
                // Keep this aligned with CLI preset semantics: `-P default`.
                KaminparPreset::Default => {
                    m::kaminpar_create_context_by_preset_name(c"default".as_ptr())
                }
                KaminparPreset::Strong => m::kaminpar_create_strong_context(),
                KaminparPreset::TeraPart => m::kaminpar_create_terapart_context(),
                KaminparPreset::LargeK => m::kaminpar_create_largek_context(),
                KaminparPreset::VCycle => m::kaminpar_create_vcycle_context(false),
            }
        };
        assert!(!ctx.is_null(), "KaMinPar failed to create context");

        let ptr = unsafe { m::kaminpar_create(num_threads, ctx) };
        assert!(!ptr.is_null(), "KaMinPar failed to create partitioner");

        KaMinParPartitioner { ctx, ptr }
    }

    pub fn set_output_level(&mut self, output_level: KaminparOutputLevel) {
        unsafe { m::kaminpar_set_output_level(self.ptr, output_level.as_raw()) }
    }

    pub fn copy_graph(
        &mut self,
        xadj: &[KaminparEdgeId],
        adjncy: &[KaminparNodeId],
        vwgt: Option<&[KaminparNodeWeight]>,
        adjwgt: Option<&[KaminparEdgeWeight]>,
    ) {
        assert_ne!(xadj.len(), 0);
        assert_eq!(adjncy.len(), *xadj.last().unwrap() as usize);

        let n: KaminparNodeId = (xadj.len() - 1).try_into().unwrap();
        if let Some(vwgt) = vwgt {
            assert_eq!(vwgt.len(), n as usize);
        }
        if let Some(adjwgt) = adjwgt {
            assert_eq!(adjwgt.len(), adjncy.len());
        }

        let vwgt_ptr = if let Some(vwgt) = vwgt {
            vwgt.as_ptr()
        } else {
            ptr::null()
        };
        let adjwgt_ptr = if let Some(adjwgt) = adjwgt {
            adjwgt.as_ptr()
        } else {
            ptr::null()
        };

        unsafe {
            m::kaminpar_copy_graph(
                self.ptr,
                n,
                xadj.as_ptr(),
                adjncy.as_ptr(),
                vwgt_ptr,
                adjwgt_ptr,
            )
        }
    }

    pub fn partition_with_epsilon(
        &mut self,
        k: KaminparBlockId,
        epsilon: f64,
        partition: &mut [KaminparBlockId],
    ) -> KaminparEdgeWeight {
        unsafe {
            m::kaminpar_compute_partition_with_epsilon(self.ptr, k, epsilon, partition.as_mut_ptr())
        }
    }
}

impl Drop for KaMinParPartitioner {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                m::kaminpar_free(self.ptr);
            }
            if !self.ctx.is_null() {
                m::kaminpar_context_free(self.ctx);
            }
        }
    }
}

impl<'a> KaMinParGraph<'a> {
    pub fn partition_with_epsilon(
        &mut self,
        n_parts: KaminparBlockId,
        params: KaminparParams,
    ) -> (Vec<KaminparBlockId>, KaminparEdgeWeight) {
        let mut kaminpar = KaMinParPartitioner::new(params.num_threads, params.preset);
        kaminpar.set_output_level(params.output_level);
        kaminpar.copy_graph(
            self.xadj,
            self.adjncy,
            self.vwgt.as_deref(),
            self.adjwgt.as_deref(),
        );

        let mut part = vec![0; self.xadj.len() - 1];
        let edge_cut = kaminpar.partition_with_epsilon(n_parts, params.epsilon, &mut part);
        (part, edge_cut)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        KaHIPGraph, KaMinParGraph, KahipMode, KahipParams, KaminparBlockId, KaminparEdgeId,
        KaminparNodeId, KaminparParams, KaminparPreset,
    };

    #[test]
    fn test_kahip() {
        let mut xadj = vec![0, 2, 5, 7, 9, 12];
        let mut adjncy = vec![1, 4, 0, 2, 4, 1, 3, 2, 4, 0, 1, 3];

        let (part, edgcut) = KaHIPGraph::new(&mut xadj, &mut adjncy).partition(
            2,
            KahipParams {
                seed: 1234,
                mode: KahipMode::Eco,
                ..KahipParams::default()
            },
        );

        assert_eq!(part, [0, 0, 1, 1, 0]);
        assert_eq!(edgcut, 2);
    }

    #[test]
    fn test_kaminpar() {
        let mut xadj: Vec<KaminparEdgeId> = vec![0, 2, 5, 7, 9, 12];
        let mut adjncy: Vec<KaminparNodeId> = vec![1, 4, 0, 2, 4, 1, 3, 2, 4, 0, 1, 3];

        let (part, edgecut) = KaMinParGraph::new(&mut xadj, &mut adjncy).partition_with_epsilon(
            2,
            KaminparParams {
                preset: KaminparPreset::Default,
                ..KaminparParams::default()
            },
        );

        assert_eq!(part.len(), xadj.len() - 1);
        assert!(part.iter().all(|&p: &KaminparBlockId| p < 2));
        assert!(edgecut >= 0);
    }

    #[test]
    fn test_kahip_reduced_nd_ordering() {
        let mut xadj = vec![0, 2, 5, 7, 9, 12];
        let mut adjncy = vec![1, 4, 0, 2, 4, 1, 3, 2, 4, 0, 1, 3];

        let mut ordering =
            KaHIPGraph::new(&mut xadj, &mut adjncy).reduced_nd_ordering(true, 1234, KahipMode::Eco);

        assert_eq!(ordering.len(), xadj.len() - 1);
        ordering.sort_unstable();
        assert_eq!(ordering, [0, 1, 2, 3, 4]);
    }
}
