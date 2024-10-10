// Copyright (c) Meta Platforms, Inc. and affiliates.

// This software may be used and distributed according to the terms of the
// GNU General Public License version 2.
mod bpf_skel;
mod layer_core_growth;
mod stats;

pub use bpf_skel::*;
pub mod bpf_intf;
use core::ffi::CStr;
use stats::LayerStats;
use stats::StatsReq;
use stats::StatsRes;
use stats::SysStats;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::mem::MaybeUninit;
use std::ops::Sub;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::ThreadId;
use std::time::Duration;
use std::time::Instant;

use ::fb_procfs as procfs;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bitvec::prelude::*;
use clap::Parser;
use clap::ValueEnum;
use crossbeam::channel::RecvTimeoutError;
use layer_core_growth::LayerGrowthAlgo;
use libbpf_rs::skel::OpenSkel;
use libbpf_rs::skel::Skel;
use libbpf_rs::skel::SkelBuilder;
use libbpf_rs::MapCore as _;
use libbpf_rs::OpenObject;
use log::debug;
use log::info;
use log::trace;
use log::warn;
use scx_stats::prelude::*;
use scx_utils::compat;
use scx_utils::init_libbpf_logging;
use scx_utils::ravg::ravg_read;
use scx_utils::Cache;
use scx_utils::Core;
use scx_utils::CoreType;
use scx_utils::LoadAggregator;
use scx_utils::Topology;
use serde::Deserialize;
use serde::Serialize;

const RAVG_FRAC_BITS: u32 = bpf_intf::ravg_consts_RAVG_FRAC_BITS;
const MAX_CPUS: usize = bpf_intf::consts_MAX_CPUS as usize;
const MAX_PATH: usize = bpf_intf::consts_MAX_PATH as usize;
const MAX_COMM: usize = bpf_intf::consts_MAX_COMM as usize;
const MAX_LAYER_WEIGHT: u32 = bpf_intf::consts_MAX_LAYER_WEIGHT;
const MIN_LAYER_WEIGHT: u32 = bpf_intf::consts_MIN_LAYER_WEIGHT;
const DEFAULT_LAYER_WEIGHT: u32 = bpf_intf::consts_DEFAULT_LAYER_WEIGHT;
const MAX_LAYER_MATCH_ORS: usize = bpf_intf::consts_MAX_LAYER_MATCH_ORS as usize;
const MAX_LAYERS: usize = bpf_intf::consts_MAX_LAYERS as usize;
const USAGE_HALF_LIFE: u32 = bpf_intf::consts_USAGE_HALF_LIFE;
const USAGE_HALF_LIFE_F64: f64 = USAGE_HALF_LIFE as f64 / 1_000_000_000.0;
const NR_GSTATS: usize = bpf_intf::global_stat_idx_NR_GSTATS as usize;
const NR_LSTATS: usize = bpf_intf::layer_stat_idx_NR_LSTATS as usize;
const NR_LAYER_MATCH_KINDS: usize = bpf_intf::layer_match_kind_NR_LAYER_MATCH_KINDS as usize;
const CORE_CACHE_LEVEL: u32 = 2;

#[rustfmt::skip]
lazy_static::lazy_static! {
    static ref NR_POSSIBLE_CPUS: usize = libbpf_rs::num_possible_cpus().unwrap();
    static ref USAGE_DECAY: f64 = 0.5f64.powf(1.0 / USAGE_HALF_LIFE_F64);
    static ref EXAMPLE_CONFIG: LayerConfig =
	LayerConfig {
            specs: vec![
		LayerSpec {
                    name: "batch".into(),
                    comment: Some("tasks under system.slice or tasks with nice value > 0".into()),
                    matches: vec![
			vec![LayerMatch::CgroupPrefix("system.slice/".into())],
			vec![LayerMatch::NiceAbove(0)],
                    ],
                    kind: LayerKind::Confined {
			cpus_range: Some((0, 16)),
			util_range: (0.8, 0.9),
			min_exec_us: 1000,
			yield_ignore: 0.0,
			preempt: false,
			preempt_first: false,
			exclusive: false,
			idle_smt: false,
                        slice_us: 20000,
                        weight: DEFAULT_LAYER_WEIGHT,
                        growth_algo: LayerGrowthAlgo::Sticky,
			perf: 1024,
			nodes: vec![],
			llcs: vec![],
                    },
		},
		LayerSpec {
                    name: "immediate".into(),
                    comment: Some("tasks under workload.slice with nice value < 0".into()),
                    matches: vec![vec![
			LayerMatch::CgroupPrefix("workload.slice/".into()),
			LayerMatch::NiceBelow(0),
                    ]],
                    kind: LayerKind::Open {
			min_exec_us: 100,
			yield_ignore: 0.25,
			preempt: true,
			preempt_first: false,
			exclusive: true,
			idle_smt: false,
                        slice_us: 20000,
                        weight: DEFAULT_LAYER_WEIGHT,
                        growth_algo: LayerGrowthAlgo::Sticky,
			perf: 1024,
			nodes: vec![],
			llcs: vec![],
                    },
		},
		LayerSpec {
                    name: "stress-ng".into(),
                    comment: Some("stress-ng test layer".into()),
                    matches: vec![vec![
			LayerMatch::CommPrefix("stress-ng".into()),
                    ],
                    vec![
			LayerMatch::PcommPrefix("stress-ng".into()),
                    ]],
                    kind: LayerKind::Confined {
			cpus_range: None,
			min_exec_us: 800,
			yield_ignore: 0.0,
			util_range: (0.2, 0.8),
			preempt: true,
			preempt_first: false,
			exclusive: false,
			idle_smt: false,
                        slice_us: 800,
                        weight: DEFAULT_LAYER_WEIGHT,
                        growth_algo: LayerGrowthAlgo::Topo,
			perf: 1024,
			nodes: vec![],
			llcs: vec![],
                    },
		},
		LayerSpec {
                    name: "normal".into(),
                    comment: Some("the rest".into()),
                    matches: vec![vec![]],
                    kind: LayerKind::Grouped {
			cpus_range: None,
			util_range: (0.5, 0.6),
			min_exec_us: 200,
			yield_ignore: 0.0,
			preempt: false,
			preempt_first: false,
			exclusive: false,
			idle_smt: false,
                        slice_us: 20000,
                        weight: DEFAULT_LAYER_WEIGHT,
                        growth_algo: LayerGrowthAlgo::Linear,
			perf: 1024,
			nodes: vec![],
			llcs: vec![],
                    },
		},
            ],
	};
}

/// scx_layered: A highly configurable multi-layer sched_ext scheduler
///
/// scx_layered allows classifying tasks into multiple layers and applying
/// different scheduling policies to them. The configuration is specified in
/// json and composed of two parts - matches and policies.
///
/// Matches
/// =======
///
/// Whenever a task is forked or its attributes are changed, the task goes
/// through a series of matches to determine the layer it belongs to. A
/// match set is composed of OR groups of AND blocks. An example:
///
///   "matches": [
///     [
///       {
///         "CgroupPrefix": "system.slice/"
///       }
///     ],
///     [
///       {
///         "CommPrefix": "fbagent"
///       },
///       {
///         "NiceAbove": 0
///       }
///     ]
///   ],
///
/// The outer array contains the OR groups and the inner AND blocks, so the
/// above matches:
///
/// - Tasks which are in the cgroup sub-hierarchy under "system.slice".
///
/// - Or tasks whose comm starts with "fbagent" and have a nice value > 0.
///
/// Currently, the following matches are supported:
///
/// - CgroupPrefix: Matches the prefix of the cgroup that the task belongs
///   to. As this is a string match, whether the pattern has the trailing
///   '/' makes a difference. For example, "TOP/CHILD/" only matches tasks
///   which are under that particular cgroup while "TOP/CHILD" also matches
///   tasks under "TOP/CHILD0/" or "TOP/CHILD1/".
///
/// - CommPrefix: Matches the task's comm prefix.
///
/// - PcommPrefix: Matches the task's thread group leader's comm prefix.
///
/// - NiceAbove: Matches if the task's nice value is greater than the
///   pattern.
///
/// - NiceBelow: Matches if the task's nice value is smaller than the
///   pattern.
///
/// - NiceEquals: Matches if the task's nice value is exactly equal to
///   the pattern.
///
/// - UIDEquals: Matches if the task's effective user id matches the value
///
/// - GIDEquals: Matches if the task's effective group id matches the value.
///
/// - PIDEquals: Matches if the task's pid matches the value.
///
/// - PPIDEquals: Matches if the task's ppid matches the value.
///
/// - TGIDEquals: Matches if the task's tgid matches the value.
///
/// While there are complexity limitations as the matches are performed in
/// BPF, it is straightforward to add more types of matches.
///
/// Policies
/// ========
///
/// The following is an example policy configuration for a layer.
///
///   "kind": {
///     "Confined": {
///       "cpus_range": [1, 8],
///       "util_range": [0.8, 0.9]
///     }
///   }
///
/// It's of "Confined" kind, which tries to concentrate the layer's tasks
/// into a limited number of CPUs. In the above case, the number of CPUs
/// assigned to the layer is scaled between 1 and 8 so that the per-cpu
/// utilization is kept between 80% and 90%. If the CPUs are loaded higher
/// than 90%, more CPUs are allocated to the layer. If the utilization drops
/// below 80%, the layer loses CPUs.
///
/// Currently, the following policy kinds are supported:
///
/// - Confined: Tasks are restricted to the allocated CPUs. The number of
///   CPUs allocated is modulated to keep the per-CPU utilization in
///   "util_range". The range can optionally be restricted with the
///   "cpus_range" property.
///
/// - Grouped: Similar to Confined but tasks may spill outside if there are
///   idle CPUs outside the allocated ones.
///
/// - Open: Prefer the CPUs which are not occupied by Confined or Grouped
///   layers. Tasks in this group will spill into occupied CPUs if there are
///   no unoccupied idle CPUs.
///
/// All layers take the following options:
///
/// - min_exec_us: Minimum execution time in microseconds. Whenever a task
///   is scheduled in, this is the minimum CPU time that it's charged no
///   matter how short the actual execution time may be.
///
/// - yield_ignore: Yield ignore ratio. If 0.0, yield(2) forfeits a whole
///   execution slice. 0.25 yields three quarters of an execution slice and
///   so on. If 1.0, yield is completely ignored.
///
/// - preempt: If true, tasks in the layer will preempt tasks which belong
///   to other non-preempting layers when no idle CPUs are available.
///
/// - preempt_first: If true, tasks in the layer will try to preempt tasks
///   in their previous CPUs before trying to find idle CPUs.
///
/// - exclusive: If true, tasks in the layer will occupy the whole core. The
///   other logical CPUs sharing the same core will be kept idle. This isn't
///   a hard guarantee, so don't depend on it for security purposes.
///
/// - slice_us: Scheduling slice duration in microseconds.
///
/// - weight: Weight of the layer, which is a range from 1 to 10000 with a
///   default of 100. Layer weights are used during contention to prevent
///   starvation across layers. Weights are used in combination with
///   utilization to determine the infeasible adjusted weight with higher
///   weights having a larger adjustment in adjusted utilization.
///
/// - idle_smt: When selecting an idle CPU for task task migration use
///   only idle SMT CPUs. The default is to select any idle cpu.
///
/// - growth_algo: When a layer is allocated new CPUs different algorithms can
///   be used to determine which CPU should be allocated next. The default
///   algorithm is a "sticky" algorithm that attempts to spread layers evenly
///   across cores.
///
/// - perf: CPU performance target. 0 means no configuration. A value
///   between 1 and 1024 indicates the performance level CPUs running tasks
///   in this layer are configured to using scx_bpf_cpuperf_set().
///
/// - nodes: If set the layer will use the set of NUMA nodes for scheduling
///   decisions. If unset then all available NUMA nodes will be used. If the
///   llcs value is set the cpuset of NUMA nodes will be or'ed with the LLC
///   config.
///
/// - llcs: If set the layer will use the set of LLCs (last level caches)
///   for scheduling decisions. If unset then all LLCs will be used. If
///   the nodes value is set the cpuset of LLCs will be or'ed with the nodes
///   config.
///
///
/// Similar to matches, adding new policies and extending existing ones
/// should be relatively straightforward.
///
/// Configuration example and running scx_layered
/// =============================================
///
/// An scx_layered config is composed of layer configs. A layer config is
/// composed of a name, a set of matches, and a policy block. Running the
/// following will write an example configuration into example.json.
///
///   $ scx_layered -e example.json
///
/// Note that the last layer in the configuration must have an empty match set
/// as a catch-all for tasks which haven't been matched into previous layers.
///
/// The configuration can be specified in multiple json files and
/// command line arguments, which are concatenated in the specified
/// order. Each must contain valid layer configurations.
///
/// By default, an argument to scx_layered is interpreted as a JSON string. If
/// the argument is a pointer to a JSON file, it should be prefixed with file:
/// or f: as follows:
///
///   $ scx_layered file:example.json
///   ...
///   $ scx_layered f:example.json
///
/// Monitoring Statistics
/// =====================
///
/// Run with `--stats INTERVAL` to enable stats monitoring. There is
/// also an scx_stat server listening on /var/run/scx/root/stat that can
/// be monitored by running `scx_layered --monitor INTERVAL` separately.
///
///   ```bash
///   $ scx_layered --monitor 1
///   tot= 117909 local=86.20 open_idle= 0.21 affn_viol= 1.37 proc=6ms
///   busy= 34.2 util= 1733.6 load=  21744.1 fallback_cpu=  1
///     batch    : util/frac=   11.8/  0.7 load/frac=     29.7:  0.1 tasks=  2597
///                tot=   3478 local=67.80 open_idle= 0.00 preempt= 0.00 affn_viol= 0.00
///                cpus=  2 [  2,  2] 04000001 00000000
///     immediate: util/frac= 1218.8/ 70.3 load/frac=  21399.9: 98.4 tasks=  1107
///                tot=  68997 local=90.57 open_idle= 0.26 preempt= 9.36 affn_viol= 0.00
///                cpus= 50 [ 50, 50] fbfffffe 000fffff
///     normal   : util/frac=  502.9/ 29.0 load/frac=    314.5:  1.4 tasks=  3512
///                tot=  45434 local=80.97 open_idle= 0.16 preempt= 0.00 affn_viol= 3.56
///                cpus= 50 [ 50, 50] fbfffffe 000fffff
///   ```
///
/// Global statistics: see [`SysStats`]
///
/// Per-layer statistics: see [`LayerStats`]
///
#[derive(Debug, Parser)]
#[command(verbatim_doc_comment)]
struct Opts {
    /// Scheduling slice duration in microseconds.
    #[clap(short = 's', long, default_value = "20000")]
    slice_us: u64,

    /// Maximum consecutive execution time in microseconds. A task may be
    /// allowed to keep executing on a CPU for this long. Note that this is
    /// the upper limit and a task may have to moved off the CPU earlier. 0
    /// indicates default - 20 * slice_us.
    #[clap(short = 'M', long, default_value = "0")]
    max_exec_us: u64,

    /// Scheduling interval in seconds.
    #[clap(short = 'i', long, default_value = "0.1")]
    interval: f64,

    /// ***DEPRECATED*** Disable load-fraction based max layer CPU limit.
    /// recommended.
    #[clap(short = 'n', long, default_value = "false")]
    no_load_frac_limit: bool,

    /// Exit debug dump buffer length. 0 indicates default.
    #[clap(long, default_value = "0")]
    exit_dump_len: u32,

    /// Enable verbose output, including libbpf details. Specify multiple
    /// times to increase verbosity.
    #[clap(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Disable topology awareness. When enabled, the "nodes" and "llcs" settings on
    /// a layer are ignored.
    #[clap(short = 't', long)]
    disable_topology: bool,

    /// Enable cross NUMA preemption.
    #[clap(long)]
    xnuma_preemption: bool,

    /// Write example layer specifications into the file and exit.
    #[clap(short = 'e', long)]
    example: Option<String>,

    /// Disables preemption if the weighted load fraction of a layer (load_frac_adj) exceeds the
    /// threshold. The default is disabled (0.0).
    #[clap(long, default_value = "0.0")]
    layer_preempt_weight_disable: f64,

    /// Disables layer growth if the weighted load fraction of a layer (load_frac_adj) exceeds the
    /// threshold. The default is disabled (0.0).
    #[clap(long, default_value = "0.0")]
    layer_growth_weight_disable: f64,

    /// When iterating over layer DSQs use the weight of the layer for iteration
    /// order. The default iteration order is semi-random except when topology
    /// awareness is disabled.
    #[clap(long, value_enum, default_value = "linear")]
    dsq_iter_algo: DsqIterAlgo,

    /// Enable stats monitoring with the specified interval.
    #[clap(long)]
    stats: Option<f64>,

    /// Run in stats monitoring mode with the specified interval. Scheduler
    /// is not launched.
    #[clap(long)]
    monitor: Option<f64>,

    /// DEPRECATED: Enable output of stats in OpenMetrics format instead of via
    /// log macros.  This option is useful if you want to collect stats in some
    /// monitoring database like prometheseus.
    #[clap(short = 'o', long)]
    open_metrics_format: bool,

    /// Run with example layer specifications (useful for e.g. CI pipelines)
    #[clap(long)]
    run_example: bool,

    /// Show descriptions for statistics.
    #[clap(long)]
    help_stats: bool,

    /// Layer specification. See --help.
    specs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum LayerMatch {
    CgroupPrefix(String),
    CommPrefix(String),
    PcommPrefix(String),
    NiceAbove(i32),
    NiceBelow(i32),
    NiceEquals(i32),
    UIDEquals(u32),
    GIDEquals(u32),
    PIDEquals(u32),
    PPIDEquals(u32),
    TGIDEquals(u32),
}

#[derive(ValueEnum, Clone, Debug, Parser, PartialEq, Serialize, Deserialize)]
#[clap(rename_all = "snake_case")]
enum DsqIterAlgo {
    /// Linear starts with the first layer in the config and iterates over
    /// layers sequentially.
    Linear,
    /// Iterates from lowest weight to highest weight.
    Weight,
    /// Iterates from the highest weigh to the lowest weight.
    ReverseWeight,
    /// Per CPU semi round robin ordering.
    RoundRobin,
}

impl DsqIterAlgo {
    fn as_bpf_enum(&self) -> u32 {
        match self {
            DsqIterAlgo::Linear => bpf_intf::dsq_iter_algo_DSQ_ITER_LINEAR,
            DsqIterAlgo::Weight => bpf_intf::dsq_iter_algo_DSQ_ITER_WEIGHT,
            DsqIterAlgo::ReverseWeight => bpf_intf::dsq_iter_algo_DSQ_ITER_REVERSE_WEIGHT,
            DsqIterAlgo::RoundRobin => bpf_intf::dsq_iter_algo_DSQ_ITER_ROUND_ROBIN,
        }
    }
}

impl Default for DsqIterAlgo {
    fn default() -> Self {
        DsqIterAlgo::RoundRobin
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum LayerKind {
    Confined {
        util_range: (f64, f64),
        #[serde(default)]
        cpus_range: Option<(usize, usize)>,
        #[serde(default)]
        min_exec_us: u64,
        #[serde(default)]
        yield_ignore: f64,
        #[serde(default)]
        slice_us: u64,
        #[serde(default)]
        preempt: bool,
        #[serde(default)]
        preempt_first: bool,
        #[serde(default)]
        exclusive: bool,
        #[serde(default)]
        weight: u32,
        #[serde(default)]
        idle_smt: bool,
        #[serde(default)]
        growth_algo: LayerGrowthAlgo,
        #[serde(default)]
        perf: u64,
        #[serde(default)]
        nodes: Vec<usize>,
        #[serde(default)]
        llcs: Vec<usize>,
    },
    Grouped {
        util_range: (f64, f64),
        #[serde(default)]
        cpus_range: Option<(usize, usize)>,
        #[serde(default)]
        min_exec_us: u64,
        #[serde(default)]
        yield_ignore: f64,
        #[serde(default)]
        slice_us: u64,
        #[serde(default)]
        preempt: bool,
        #[serde(default)]
        preempt_first: bool,
        #[serde(default)]
        exclusive: bool,
        #[serde(default)]
        weight: u32,
        #[serde(default)]
        idle_smt: bool,
        #[serde(default)]
        growth_algo: LayerGrowthAlgo,
        #[serde(default)]
        perf: u64,
        #[serde(default)]
        nodes: Vec<usize>,
        #[serde(default)]
        llcs: Vec<usize>,
    },
    Open {
        #[serde(default)]
        min_exec_us: u64,
        #[serde(default)]
        yield_ignore: f64,
        #[serde(default)]
        slice_us: u64,
        #[serde(default)]
        preempt: bool,
        #[serde(default)]
        preempt_first: bool,
        #[serde(default)]
        exclusive: bool,
        #[serde(default)]
        weight: u32,
        #[serde(default)]
        idle_smt: bool,
        #[serde(default)]
        growth_algo: LayerGrowthAlgo,
        #[serde(default)]
        perf: u64,
        #[serde(default)]
        nodes: Vec<usize>,
        #[serde(default)]
        llcs: Vec<usize>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LayerSpec {
    name: String,
    comment: Option<String>,
    matches: Vec<Vec<LayerMatch>>,
    kind: LayerKind,
}

impl LayerSpec {
    fn parse(input: &str) -> Result<Vec<Self>> {
        let config: LayerConfig = if input.starts_with("f:") || input.starts_with("file:") {
            let mut f = fs::OpenOptions::new()
                .read(true)
                .open(input.split_once(':').unwrap().1)?;
            let mut content = String::new();
            f.read_to_string(&mut content)?;
            serde_json::from_str(&content)?
        } else {
            serde_json::from_str(input)?
        };
        Ok(config.specs)
    }
    fn nodes(&self) -> Vec<usize> {
        match &self.kind {
            LayerKind::Confined { nodes, .. }
            | LayerKind::Open { nodes, .. }
            | LayerKind::Grouped { nodes, .. } => nodes.clone(),
        }
    }
    fn llcs(&self) -> Vec<usize> {
        match &self.kind {
            LayerKind::Confined { llcs, .. }
            | LayerKind::Open { llcs, .. }
            | LayerKind::Grouped { llcs, .. } => llcs.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct LayerConfig {
    specs: Vec<LayerSpec>,
}

fn now_monotonic() -> u64 {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let ret = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) };
    assert!(ret == 0);
    time.tv_sec as u64 * 1_000_000_000 + time.tv_nsec as u64
}

fn read_total_cpu(reader: &procfs::ProcReader) -> Result<procfs::CpuStat> {
    reader
        .read_stat()
        .context("Failed to read procfs")?
        .total_cpu
        .ok_or_else(|| anyhow!("Could not read total cpu stat in proc"))
}

fn calc_util(curr: &procfs::CpuStat, prev: &procfs::CpuStat) -> Result<f64> {
    match (curr, prev) {
        (
            procfs::CpuStat {
                user_usec: Some(curr_user),
                nice_usec: Some(curr_nice),
                system_usec: Some(curr_system),
                idle_usec: Some(curr_idle),
                iowait_usec: Some(curr_iowait),
                irq_usec: Some(curr_irq),
                softirq_usec: Some(curr_softirq),
                stolen_usec: Some(curr_stolen),
                ..
            },
            procfs::CpuStat {
                user_usec: Some(prev_user),
                nice_usec: Some(prev_nice),
                system_usec: Some(prev_system),
                idle_usec: Some(prev_idle),
                iowait_usec: Some(prev_iowait),
                irq_usec: Some(prev_irq),
                softirq_usec: Some(prev_softirq),
                stolen_usec: Some(prev_stolen),
                ..
            },
        ) => {
            let idle_usec = curr_idle.saturating_sub(*prev_idle);
            let iowait_usec = curr_iowait.saturating_sub(*prev_iowait);
            let user_usec = curr_user.saturating_sub(*prev_user);
            let system_usec = curr_system.saturating_sub(*prev_system);
            let nice_usec = curr_nice.saturating_sub(*prev_nice);
            let irq_usec = curr_irq.saturating_sub(*prev_irq);
            let softirq_usec = curr_softirq.saturating_sub(*prev_softirq);
            let stolen_usec = curr_stolen.saturating_sub(*prev_stolen);

            let busy_usec =
                user_usec + system_usec + nice_usec + irq_usec + softirq_usec + stolen_usec;
            let total_usec = idle_usec + busy_usec + iowait_usec;
            if total_usec > 0 {
                Ok(((busy_usec as f64) / (total_usec as f64)).clamp(0.0, 1.0))
            } else {
                Ok(1.0)
            }
        }
        _ => bail!("Missing stats in cpustat"),
    }
}

fn copy_into_cstr(dst: &mut [i8], src: &str) {
    let cstr = CString::new(src).unwrap();
    let bytes = unsafe { std::mem::transmute::<&[u8], &[i8]>(cstr.as_bytes_with_nul()) };
    dst[0..bytes.len()].copy_from_slice(bytes);
}

fn nodemask_from_nodes(nodes: &Vec<usize>) -> usize {
    let mut mask = 0;
    for node in nodes {
        mask |= 1 << node;
    }
    mask
}

fn cachemask_from_llcs(llcs: &BTreeMap<usize, Cache>) -> usize {
    let mut mask = 0;
    for (_, cache) in llcs {
        mask |= 1 << cache.id();
    }
    mask
}

fn read_cpu_ctxs(skel: &BpfSkel) -> Result<Vec<bpf_intf::cpu_ctx>> {
    let mut cpu_ctxs = vec![];
    let cpu_ctxs_vec = skel
        .maps
        .cpu_ctxs
        .lookup_percpu(&0u32.to_ne_bytes(), libbpf_rs::MapFlags::ANY)
        .context("Failed to lookup cpu_ctx")?
        .unwrap();
    for cpu in 0..*NR_POSSIBLE_CPUS {
        cpu_ctxs.push(*unsafe {
            &*(cpu_ctxs_vec[cpu].as_slice().as_ptr() as *const bpf_intf::cpu_ctx)
        });
    }
    Ok(cpu_ctxs)
}

fn convert_cpu_ctxs(cpu_ctxs: Vec<bpf_intf::cpu_ctx>) -> Vec<Vec<u8>> {
    cpu_ctxs
        .into_iter()
        .map(|cpu_ctx| {
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    &cpu_ctx as *const bpf_intf::cpu_ctx as *const u8,
                    std::mem::size_of::<bpf_intf::cpu_ctx>(),
                )
            };
            bytes.to_vec()
        })
        .collect()
}

fn initialize_cpu_ctxs(skel: &BpfSkel, topo: &Topology) -> Result<()> {
    let key = (0_u32).to_ne_bytes();
    let mut cpu_ctxs: Vec<bpf_intf::cpu_ctx> = vec![];
    let cpu_ctxs_vec = skel
        .maps
        .cpu_ctxs
        .lookup_percpu(&key, libbpf_rs::MapFlags::ANY)
        .context("Failed to lookup cpu_ctx")?
        .unwrap();

    for cpu in 0..*NR_POSSIBLE_CPUS {
        cpu_ctxs.push(*unsafe {
            &*(cpu_ctxs_vec[cpu].as_slice().as_ptr() as *const bpf_intf::cpu_ctx)
        });

        let topo_cpu = topo.cpus().get(&cpu).unwrap();
        let is_big = topo_cpu.core_type == CoreType::Big { turbo: true };
        cpu_ctxs[cpu].is_big = is_big;
    }

    skel.maps
        .cpu_ctxs
        .update_percpu(&key, &convert_cpu_ctxs(cpu_ctxs), libbpf_rs::MapFlags::ANY)
        .context("Failed to update cpu_ctx")?;

    Ok(())
}

#[derive(Clone, Debug)]
struct BpfStats {
    gstats: Vec<u64>,
    lstats: Vec<Vec<u64>>,
    lstats_sums: Vec<u64>,
}

impl BpfStats {
    fn read(cpu_ctxs: &[bpf_intf::cpu_ctx], nr_layers: usize) -> Self {
        let mut gstats = vec![0u64; NR_GSTATS];
        let mut lstats = vec![vec![0u64; NR_LSTATS]; nr_layers];

        for cpu in 0..*NR_POSSIBLE_CPUS {
            for stat in 0..NR_GSTATS {
                gstats[stat] += cpu_ctxs[cpu].gstats[stat];
            }
            for layer in 0..nr_layers {
                for stat in 0..NR_LSTATS {
                    lstats[layer][stat] += cpu_ctxs[cpu].lstats[layer][stat];
                }
            }
        }

        let mut lstats_sums = vec![0u64; NR_LSTATS];
        for layer in 0..nr_layers {
            for stat in 0..NR_LSTATS {
                lstats_sums[stat] += lstats[layer][stat];
            }
        }

        Self {
            gstats,
            lstats,
            lstats_sums,
        }
    }
}

impl<'a, 'b> Sub<&'b BpfStats> for &'a BpfStats {
    type Output = BpfStats;

    fn sub(self, rhs: &'b BpfStats) -> BpfStats {
        let vec_sub = |l: &[u64], r: &[u64]| l.iter().zip(r.iter()).map(|(l, r)| *l - *r).collect();
        BpfStats {
            gstats: vec_sub(&self.gstats, &rhs.gstats),
            lstats: self
                .lstats
                .iter()
                .zip(rhs.lstats.iter())
                .map(|(l, r)| vec_sub(l, r))
                .collect(),
            lstats_sums: vec_sub(&self.lstats_sums, &rhs.lstats_sums),
        }
    }
}

#[derive(Clone, Debug)]
struct Stats {
    nr_layers: usize,
    at: Instant,

    nr_layer_tasks: Vec<usize>,

    nr_nodes: usize,
    total_load: f64,
    layer_loads: Vec<f64>,

    // infeasible stats
    layer_load_sums: Vec<f64>,
    total_load_sum: f64,
    layer_dcycle_sums: Vec<f64>,
    total_dcycle_sum: f64,

    total_util: f64, // Running AVG of sum of layer_utils
    layer_utils: Vec<f64>,
    prev_layer_cycles: Vec<u64>,

    cpu_busy: f64, // Read from /proc, maybe higher than total_util
    prev_total_cpu: procfs::CpuStat,

    bpf_stats: BpfStats,
    prev_bpf_stats: BpfStats,

    processing_dur: Duration,
    prev_processing_dur: Duration,

    layer_slice_us: Vec<u64>,
}

impl Stats {
    fn read_layer_loads(skel: &mut BpfSkel, nr_layers: usize) -> (f64, Vec<f64>) {
        let now_mono = now_monotonic();
        let layer_loads: Vec<f64> = skel
            .maps
            .bss_data
            .layers
            .iter()
            .take(nr_layers)
            .map(|layer| {
                let rd = &layer.load_rd;
                ravg_read(
                    rd.val,
                    rd.val_at,
                    rd.old,
                    rd.cur,
                    now_mono,
                    USAGE_HALF_LIFE,
                    RAVG_FRAC_BITS,
                )
            })
            .collect();
        (layer_loads.iter().sum(), layer_loads)
    }

    fn read_layer_cycles(cpu_ctxs: &[bpf_intf::cpu_ctx], nr_layers: usize) -> Vec<u64> {
        let mut layer_cycles = vec![0u64; nr_layers];

        for cpu in 0..*NR_POSSIBLE_CPUS {
            for layer in 0..nr_layers {
                layer_cycles[layer] += cpu_ctxs[cpu].layer_cycles[layer];
            }
        }

        layer_cycles
    }

    fn new(skel: &mut BpfSkel, proc_reader: &procfs::ProcReader) -> Result<Self> {
        let nr_layers = skel.maps.rodata_data.nr_layers as usize;
        let cpu_ctxs = read_cpu_ctxs(skel)?;
        let bpf_stats = BpfStats::read(&cpu_ctxs, nr_layers);
        let nr_nodes = skel.maps.rodata_data.nr_nodes as usize;

        Ok(Self {
            at: Instant::now(),
            nr_layers,

            nr_layer_tasks: vec![0; nr_layers],

            nr_nodes,
            total_load: 0.0,
            layer_loads: vec![0.0; nr_layers],

            layer_load_sums: vec![0.0, nr_layers as f64],
            total_load_sum: 0.0,
            layer_dcycle_sums: vec![0.0, nr_layers as f64],
            total_dcycle_sum: 0.0,

            total_util: 0.0,
            layer_utils: vec![0.0; nr_layers],
            prev_layer_cycles: Self::read_layer_cycles(&cpu_ctxs, nr_layers),

            cpu_busy: 0.0,
            prev_total_cpu: read_total_cpu(&proc_reader)?,

            bpf_stats: bpf_stats.clone(),
            prev_bpf_stats: bpf_stats,

            processing_dur: Default::default(),
            prev_processing_dur: Default::default(),

            layer_slice_us: vec![0; nr_layers],
        })
    }

    fn refresh(
        &mut self,
        skel: &mut BpfSkel,
        proc_reader: &procfs::ProcReader,
        load_agg: &mut LoadAggregator,
        now: Instant,
        cur_processing_dur: Duration,
    ) -> Result<()> {
        let elapsed = now.duration_since(self.at).as_secs_f64() as f64;
        let cpu_ctxs = read_cpu_ctxs(skel)?;

        let nr_layer_tasks: Vec<usize> = skel
            .maps
            .bss_data
            .layers
            .iter()
            .take(self.nr_layers)
            .map(|layer| layer.nr_tasks as usize)
            .collect();
        let layer_weights: Vec<usize> = skel
            .maps
            .bss_data
            .layers
            .iter()
            .take(self.nr_layers)
            .map(|layer| layer.weight as usize)
            .collect();

        let layer_slice_us: Vec<u64> = skel
            .maps
            .bss_data
            .layers
            .iter()
            .take(self.nr_layers)
            .map(|layer| layer.slice_ns / 1000 as u64)
            .collect();

        let (total_load, layer_loads) = Self::read_layer_loads(skel, self.nr_layers);

        let cur_layer_cycles = Self::read_layer_cycles(&cpu_ctxs, self.nr_layers);
        cur_layer_cycles
            .iter()
            .zip(layer_weights)
            .enumerate()
            .for_each(|(layer_idx, (usage, weight))| {
                let mut load = 0.0;
                if self.prev_layer_cycles[layer_idx] > 0 {
                    load = (*usage - self.prev_layer_cycles[layer_idx]) as f64;
                }
                let _ = load_agg.record_dom_load(layer_idx, weight, load as f64);
            });
        let cur_layer_utils: Vec<f64> = cur_layer_cycles
            .iter()
            .zip(self.prev_layer_cycles.iter())
            .map(|(cur, prev)| (cur - prev) as f64 / 1_000_000_000.0 / elapsed)
            .collect();
        let layer_utils: Vec<f64> = cur_layer_utils
            .iter()
            .zip(self.layer_utils.iter())
            .map(|(cur, prev)| {
                let decay = USAGE_DECAY.powf(elapsed);
                prev * decay + cur * (1.0 - decay)
            })
            .collect();

        let load_ledger = load_agg.calculate();
        let cur_total_cpu = read_total_cpu(proc_reader)?;
        let cpu_busy = calc_util(&cur_total_cpu, &self.prev_total_cpu)?;

        let cur_bpf_stats = BpfStats::read(&cpu_ctxs, self.nr_layers);
        let bpf_stats = &cur_bpf_stats - &self.prev_bpf_stats;

        let processing_dur = cur_processing_dur
            .checked_sub(self.prev_processing_dur)
            .unwrap();

        *self = Self {
            at: now,
            nr_layers: self.nr_layers,

            nr_layer_tasks,

            nr_nodes: self.nr_nodes,
            total_load,
            layer_loads,

            total_load_sum: load_ledger.global_load_sum(),
            layer_load_sums: load_ledger.dom_load_sums().to_vec(),
            total_dcycle_sum: load_ledger.global_dcycle_sum(),
            layer_dcycle_sums: load_ledger.dom_dcycle_sums().to_vec(),

            total_util: layer_utils.iter().sum(),
            layer_utils: layer_utils.try_into().unwrap(),
            prev_layer_cycles: cur_layer_cycles,

            cpu_busy,
            prev_total_cpu: cur_total_cpu,

            bpf_stats,
            prev_bpf_stats: cur_bpf_stats,

            processing_dur,
            prev_processing_dur: cur_processing_dur,

            layer_slice_us,
        };
        Ok(())
    }
}

#[derive(Debug, Default)]
struct UserExitInfo {
    kind: i32,
    reason: Option<String>,
    msg: Option<String>,
}

impl UserExitInfo {
    fn read(bpf_uei: &types::user_exit_info) -> Result<Self> {
        let kind = unsafe { std::ptr::read_volatile(&bpf_uei.kind as *const _) };

        let (reason, msg) = if kind != 0 {
            (
                Some(
                    unsafe { CStr::from_ptr(bpf_uei.reason.as_ptr() as *const _) }
                        .to_str()
                        .context("Failed to convert reason to string")?
                        .to_string(),
                )
                .filter(|s| !s.is_empty()),
                Some(
                    unsafe { CStr::from_ptr(bpf_uei.msg.as_ptr() as *const _) }
                        .to_str()
                        .context("Failed to convert msg to string")?
                        .to_string(),
                )
                .filter(|s| !s.is_empty()),
            )
        } else {
            (None, None)
        };

        Ok(Self { kind, reason, msg })
    }

    fn exited(bpf_uei: &types::user_exit_info) -> Result<bool> {
        Ok(Self::read(bpf_uei)?.kind != 0)
    }

    fn report(&self) -> Result<()> {
        let why = match (&self.reason, &self.msg) {
            (Some(reason), None) => format!("{}", reason),
            (Some(reason), Some(msg)) => format!("{} ({})", reason, msg),
            _ => "".into(),
        };

        match self.kind {
            0 => Ok(()),
            etype => {
                if etype != 64 {
                    bail!("EXIT: kind={} {}", etype, why);
                } else {
                    info!("EXIT: {}", why);
                    Ok(())
                }
            }
        }
    }
}

#[derive(Debug)]
/// `CpuPool` represents the CPU core and logical CPU topology within the system.
/// It manages the mapping and availability of physical and logical cores, including
/// how resources are allocated for tasks across the available CPUs.
struct CpuPool {
    /// The number of physical cores available on the system.
    nr_cores: usize,

    /// The total number of logical CPUs (including SMT threads).
    /// This can be larger than `nr_cores` if SMT is enabled,
    /// where each physical core may have a couple logical cores.
    nr_cpus: usize,

    /// A bit mask representing all online logical cores.
    /// Each bit corresponds to whether a logical core (CPU) is online and available
    /// for processing tasks.
    all_cpus: BitVec,

    /// A vector of bit masks, each representing the mapping between
    /// physical cores and the logical cores that run on them.
    /// The index in the vector represents the physical core, and each bit in the
    /// corresponding `BitVec` represents whether a logical core belongs to that physical core.
    core_cpus: Vec<BitVec>,

    /// A vector that maps the index of each logical core to the sibling core.
    /// This represents the "next sibling" core within a package in systems that support SMT.
    /// The sibling core is the other logical core that shares the physical resources
    /// of the same physical core.
    sibling_cpu: Vec<i32>,

    /// A list of physical core IDs.
    /// Each entry in this vector corresponds to a unique physical core.
    cpu_core: Vec<usize>,

    /// A bit mask representing all available physical cores.
    /// Each bit corresponds to whether a physical core is available for task scheduling.
    available_cores: BitVec,

    /// The ID of the first physical core in the system.
    /// This core is often used as a default for initializing tasks.
    first_cpu: usize,

    /// The ID of the next free CPU or the fallback CPU if none are available.
    /// This is used to allocate resources when a task needs to be assigned to a core.
    fallback_cpu: usize,

    /// A mapping of node IDs to last-level cache (LLC) IDs.
    /// The map allows for the identification of which last-level cache
    /// corresponds to each CPU based on its core topology.
    core_topology_to_id: BTreeMap<(usize, usize, usize), usize>,
}

impl CpuPool {
    fn new(topo: &Topology) -> Result<Self> {
        if *NR_POSSIBLE_CPUS > MAX_CPUS {
            bail!(
                "NR_POSSIBLE_CPUS {} > MAX_CPUS {}",
                *NR_POSSIBLE_CPUS,
                MAX_CPUS
            );
        }

        let mut cpu_to_cache = vec![]; // (cpu_id, Option<cache_id>)
        let mut cache_ids = BTreeSet::<usize>::new();
        let mut nr_offline = 0;

        // Build cpu -> cache ID mapping.
        for cpu in 0..*NR_POSSIBLE_CPUS {
            let path = format!(
                "/sys/devices/system/cpu/cpu{}/cache/index{}/id",
                cpu, CORE_CACHE_LEVEL
            );
            let id = match std::fs::read_to_string(&path) {
                Ok(val) => Some(val.trim().parse::<usize>().with_context(|| {
                    format!("Failed to parse {:?}'s content {:?}", &path, &val)
                })?),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    nr_offline += 1;
                    None
                }
                Err(e) => return Err(e).with_context(|| format!("Failed to open {:?}", &path)),
            };

            cpu_to_cache.push(id);
            if let Some(id) = id {
                cache_ids.insert(id);
            }
        }

        let nr_cpus = *NR_POSSIBLE_CPUS - nr_offline;

        // Cache IDs may have holes. Assign consecutive core IDs to existing
        // cache IDs.
        let mut cache_to_core = BTreeMap::<usize, usize>::new();
        let mut nr_cores = 0;
        for cache_id in cache_ids.iter() {
            cache_to_core.insert(*cache_id, nr_cores);
            nr_cores += 1;
        }

        // Build core -> cpumask and cpu -> core mappings.
        let mut all_cpus = bitvec![0; *NR_POSSIBLE_CPUS];
        let mut core_cpus = vec![bitvec![0; *NR_POSSIBLE_CPUS]; nr_cores];
        let mut cpu_core = vec![];

        for (cpu, cache) in cpu_to_cache.iter().enumerate().take(*NR_POSSIBLE_CPUS) {
            if let Some(cache_id) = cache {
                let core_id = cache_to_core[cache_id];
                all_cpus.set(cpu, true);
                core_cpus[core_id].set(cpu, true);
                cpu_core.push(core_id);
            }
        }

        // Build sibling_cpu[]
        let mut sibling_cpu = vec![-1i32; *NR_POSSIBLE_CPUS];
        for cpus in &core_cpus {
            let mut first = -1i32;
            for cpu in cpus.iter_ones() {
                if first < 0 {
                    first = cpu as i32;
                } else {
                    sibling_cpu[first as usize] = cpu as i32;
                    sibling_cpu[cpu as usize] = first;
                    break;
                }
            }
        }

        // Build core_topology_to_id
        let mut core_topology_to_id = BTreeMap::new();
        let mut next_topo_id: usize = 0;
        for node in topo.nodes() {
            for llc in node.llcs().values() {
                for core in llc.cores().values() {
                    core_topology_to_id
                        .insert((core.node_id, core.llc_id, core.id()), next_topo_id);
                    next_topo_id += 1;
                }
            }
        }

        info!(
            "CPUs: online/possible={}/{} nr_cores={}",
            nr_cpus, *NR_POSSIBLE_CPUS, nr_cores,
        );
        debug!("CPUs: siblings={:?}", &sibling_cpu[..nr_cpus]);

        let first_cpu = core_cpus[0].first_one().unwrap();

        let mut cpu_pool = Self {
            nr_cores,
            nr_cpus,
            all_cpus,
            core_cpus,
            sibling_cpu,
            cpu_core,
            available_cores: bitvec![1; nr_cores],
            first_cpu,
            fallback_cpu: first_cpu,
            core_topology_to_id,
        };
        cpu_pool.update_fallback_cpu();
        Ok(cpu_pool)
    }

    fn update_fallback_cpu(&mut self) {
        match self.available_cores.first_one() {
            Some(next) => self.fallback_cpu = self.core_cpus[next].first_one().unwrap(),
            None => self.fallback_cpu = self.first_cpu,
        }
    }

    fn alloc_cpus<'a>(&'a mut self, layer: &Layer) -> Option<&'a BitVec> {
        let available_cpus = self.available_cpus_in_mask(&layer.allowed_cpus);
        let available_cores = self.cpus_to_cores(&available_cpus).ok()?;

        for alloc_core in layer.core_alloc_order() {
            match available_cores.get(*alloc_core) {
                Some(bit) => {
                    if *bit {
                        self.available_cores.set(*alloc_core, false);
                        self.update_fallback_cpu();
                        return Some(&self.core_cpus[*alloc_core]);
                    }
                }
                None => {
                    continue;
                }
            }
        }
        None
    }

    fn cpus_to_cores(&self, cpus_to_match: &BitVec) -> Result<BitVec> {
        let mut cpus = cpus_to_match.clone();
        let mut cores = bitvec![0; self.nr_cores];

        while let Some(cpu) = cpus.first_one() {
            let core = self.cpu_core[cpu];

            if (self.core_cpus[core].clone() & !cpus.clone()).count_ones() != 0 {
                bail!(
                    "CPUs {} partially intersect with core {} ({})",
                    cpus_to_match,
                    core,
                    self.core_cpus[core],
                );
            }

            cpus &= !self.core_cpus[core].clone();
            cores.set(core, true);
        }

        Ok(cores)
    }

    fn free<'a>(&'a mut self, cpus_to_free: &BitVec) -> Result<()> {
        let cores = self.cpus_to_cores(cpus_to_free)?;
        if (self.available_cores.clone() & &cores).any() {
            bail!("Some of CPUs {} are already free", cpus_to_free);
        }
        self.available_cores |= cores;
        self.update_fallback_cpu();
        Ok(())
    }

    fn next_to_free<'a>(&'a self, cands: &BitVec) -> Result<Option<&'a BitVec>> {
        let last = match cands.last_one() {
            Some(ret) => ret,
            None => return Ok(None),
        };
        let core = self.cpu_core[last];
        if (self.core_cpus[core].clone() & !cands.clone()).count_ones() != 0 {
            bail!(
                "CPUs{} partially intersect with core {} ({})",
                cands,
                core,
                self.core_cpus[core]
            );
        }

        Ok(Some(&self.core_cpus[core]))
    }

    fn available_cpus_in_mask(&self, allowed_cpus: &BitVec) -> BitVec {
        let mut cpus = bitvec![0; self.nr_cpus];
        for core in self.available_cores.iter_ones() {
            let mut core_cpus = self.core_cpus[core].clone();
            core_cpus &= allowed_cpus;
            cpus |= core_cpus;
        }
        cpus
    }

    fn get_core_topological_id(&self, core: &Core) -> usize {
        *self
            .core_topology_to_id
            .get(&(core.node_id, core.llc_id, core.id()))
            .expect("unrecognised core")
    }
}

struct IteratorInterleaver<T>
where
    T: Iterator,
{
    empty: bool,
    index: usize,
    iters: Vec<T>,
}
fn layer_core_order(growth_algo: LayerGrowthAlgo, layer_idx: usize, topo: &Topology) -> Vec<usize> {
    let mut core_order = vec![];
    match growth_algo {
        LayerGrowthAlgo::Sticky => {
            let is_left = layer_idx % 2 == 0;
            let rot_by = |layer_idx, len| -> usize {
                if layer_idx <= len {
                    layer_idx
                } else {
                    layer_idx % len
                }
            };

impl<T> IteratorInterleaver<T>
where
    T: Iterator,
{
    fn new(iters: Vec<T>) -> Self {
        Self {
            empty: false,
            index: 0,
            iters,
        }
    }
}

impl<T> Iterator for IteratorInterleaver<T>
where
    T: Iterator,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<T::Item> {
        if let Some(iter) = self.iters.get_mut(self.index) {
            self.index += 1;
            if let Some(value) = iter.next() {
                self.empty = false;
                Some(value)
            } else {
                self.next()
            }
        } else {
            self.index = 0;
            if self.empty {
                None
            } else {
                self.empty = true;
                self.next()
            }
        }
    }
}

#[derive(Debug)]
struct Layer {
    name: String,
    kind: LayerKind,
    core_order: Vec<usize>,

    nr_cpus: usize,
    preempt: bool,
    can_preempt: bool,
    cpus: BitVec,
    allowed_cpus: BitVec,
}

impl Layer {
    fn new(
        idx: usize,
        cpu_pool: &CpuPool,
        name: &str,
        kind: LayerKind,
        topo: &Topology,
    ) -> Result<Self> {
        let mut cpus = bitvec![0; cpu_pool.nr_cpus];
        cpus.fill(false);
        let mut allowed_cpus = bitvec![0; cpu_pool.nr_cpus];
        let mut layer_growth_algo = LayerGrowthAlgo::Sticky;
        match &kind {
            LayerKind::Confined {
                cpus_range,
                util_range,
                nodes,
                llcs,
                growth_algo,
                ..
            } => {
                layer_growth_algo = growth_algo.clone();
                let cpus_range = cpus_range.unwrap_or((0, std::usize::MAX));
                if cpus_range.0 > cpus_range.1 || cpus_range.1 == 0 {
                    bail!("invalid cpus_range {:?}", cpus_range);
                }
                if nodes.len() == 0 && llcs.len() == 0 {
                    allowed_cpus.fill(true);
                } else {
                    // build up the cpus bitset
                    for node in topo.nodes() {
                        // first do the matching for nodes
                        if nodes.contains(&(node.id() as usize)) {
                            for (id, _cpu) in node.cpus() {
                                allowed_cpus.set(id, true);
                            }
                        }
                        // next match on any LLCs
                        for (_, llc) in node.llcs() {
                            if llcs.contains(&(llc.id() as usize)) {
                                for (id, _cpu) in llc.cpus() {
                                    allowed_cpus.set(id, true);
                                }
                            }
                        }
                    }
                }

                if util_range.0 < 0.0
                    || util_range.0 > 1.0
                    || util_range.1 < 0.0
                    || util_range.1 > 1.0
                    || util_range.0 >= util_range.1
                {
                    bail!("invalid util_range {:?}", util_range);
                }
            }
            LayerKind::Grouped {
                growth_algo,
                nodes,
                llcs,
                ..
            }
            | LayerKind::Open {
                growth_algo,
                nodes,
                llcs,
                ..
            } => {
                layer_growth_algo = growth_algo.clone();
                if nodes.len() == 0 && llcs.len() == 0 {
                    allowed_cpus.fill(true);
                } else {
                    // build up the cpus bitset
                    for node in topo.nodes() {
                        // first do the matching for nodes
                        if nodes.contains(&(node.id() as usize)) {
                            for (id, _cpu) in node.cpus() {
                                allowed_cpus.set(id, true);
                            }
                        }
                        // next match on any LLCs
                        for (_, llc) in node.llcs() {
                            if llcs.contains(&(llc.id() as usize)) {
                                for (id, _cpu) in llc.cpus() {
                                    allowed_cpus.set(id, true);
                                }
                            }
                        }
                    }
                }
            }
        }

<<<<<<< HEAD
        let layer_growth_algo = match &kind {
            LayerKind::Confined { growth_algo, .. }
            | LayerKind::Grouped { growth_algo, .. }
            | LayerKind::Open { growth_algo, .. } => growth_algo.clone(),
        };
        let preempt = match &kind {
            LayerKind::Confined { preempt, .. }
            | LayerKind::Grouped { preempt, .. }
            | LayerKind::Open { preempt, .. } => preempt.clone(),
        };

        let core_order = layer_growth_algo.layer_core_order(cpu_pool, spec, idx, topo);
        debug!(
            "layer: {} algo: {:?} core order: {:?}",
            name,
            layer_growth_algo.clone(),
            core_order
        );
        let layer_growth_algo = match &kind {
            LayerKind::Confined { growth_algo, .. }
            | LayerKind::Grouped { growth_algo, .. }
            | LayerKind::Open { growth_algo, .. } => growth_algo.clone(),
        };

        let core_order = layer_core_order(layer_growth_algo, idx, topo);

        Ok(Self {
            name: name.into(),
            kind,
            core_order,

            nr_cpus: 0,
            preempt,
            can_preempt: preempt,
            cpus,
            allowed_cpus,
        })
    }

    fn core_alloc_order(&self) -> &Vec<usize> {
        &self.core_order
    }

    fn grow_confined_or_grouped(
        &mut self,
        cpu_pool: &mut CpuPool,
        (cpus_min, cpus_max): (usize, usize),
        (_util_low, util_high): (f64, f64),
        layer_growth_weight_disable: f64,
        (layer_load, total_load): (f64, f64),
        (layer_util, _total_util): (f64, f64),
    ) -> Result<bool> {
        let nr_cpus = self.cpus.count_ones();
        if nr_cpus >= cpus_max {
            trace!("layer has {} max: {}", nr_cpus, cpus_max);
            return Ok(false);
        }

        // Do we already have enough?
        if nr_cpus >= cpus_min
            && (layer_util == 0.0 || (nr_cpus > 0 && layer_util / nr_cpus as f64 <= util_high))
        {
            return Ok(false);
        }
        if total_load > 0.0
            && layer_growth_weight_disable > 0.0
            && layer_load / total_load > layer_growth_weight_disable
        {
            trace!(
                "layer-{} needs more CPUs (util={:.3}) but is over the load fraction",
                &self.name,
                layer_util
            );
            return Ok(false);
        }

        let new_cpus = match cpu_pool.alloc_cpus(&self).clone() {
            Some(ret) => ret.clone(),
            None => {
                trace!("layer-{} can't grow, no CPUs", &self.name);
                return Ok(false);
            }
        };

        trace!(
            "layer-{} adding {} CPUs to {} CPUs",
            &self.name,
            new_cpus.count_ones(),
            nr_cpus,
        );
        self.cpus |= &new_cpus;
        self.nr_cpus = self.cpus.count_ones();
        Ok(true)
    }

    fn cpus_to_free(
        &self,
        cpu_pool: &mut CpuPool,
        (cpus_min, _cpus_max): (usize, usize),
        (util_low, util_high): (f64, f64),
        layer_growth_weight_disable: f64,
        (layer_load, total_load): (f64, f64),
        (layer_util, _total_util): (f64, f64),
    ) -> Result<Option<BitVec>> {
        let nr_cpus = self.cpus.count_ones();
        if nr_cpus <= cpus_min {
            return Ok(None);
        }
        let cpus_to_free = match cpu_pool.next_to_free(&self.cpus)? {
            Some(ret) => ret.clone(),
            None => return Ok(None),
        };

        let nr_to_free = cpus_to_free.count_ones();

        // If we'd be over the load fraction even after freeing
        // $cpus_to_free, we have to free.
        if layer_growth_weight_disable > 0.0
            && layer_load / total_load >= layer_growth_weight_disable
        {
            return Ok(Some(cpus_to_free));
        }

        if layer_util / nr_cpus as f64 >= util_low {
            return Ok(None);
        }

        // Can't shrink if losing the CPUs pushes us over @util_high.
        match nr_cpus - nr_to_free {
            0 => {
                if layer_util > 0.0 {
                    return Ok(None);
                }
            }
            nr_left => {
                if layer_util / nr_left as f64 >= util_high {
                    return Ok(None);
                }
            }
        }

        return Ok(Some(cpus_to_free));
    }

    fn shrink_confined_or_grouped(
        &mut self,
        cpu_pool: &mut CpuPool,
        cpus_range: (usize, usize),
        util_range: (f64, f64),
        layer_growth_weight_disable: f64,
        load: (f64, f64),
        util: (f64, f64),
    ) -> Result<bool> {
        match self.cpus_to_free(
            cpu_pool,
            cpus_range,
            util_range,
            layer_growth_weight_disable,
            load,
            util,
        )? {
            Some(cpus_to_free) => {
                trace!("{} freeing CPUs\n{}", self.name, &cpus_to_free);
                self.cpus &= !cpus_to_free.clone();
                cpu_pool.free(&cpus_to_free)?;
                self.nr_cpus = self.cpus.count_ones();
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn resize_confined_or_grouped(
        &mut self,
        cpu_pool: &mut CpuPool,
        cpus_range: Option<(usize, usize)>,
        util_range: (f64, f64),
        layer_growth_weight_disable: f64,
        load: (f64, f64),
        util: (f64, f64),
    ) -> Result<i64> {
        let cpus_range = cpus_range.unwrap_or((0, std::usize::MAX));
        let mut adjusted = 0;

        while self.grow_confined_or_grouped(
            cpu_pool,
            cpus_range,
            util_range,
            layer_growth_weight_disable,
            load,
            util,
        )? {
            adjusted += 1;
            trace!("{} grew, adjusted={}", &self.name, adjusted);
        }

        if adjusted == 0 {
            while self.shrink_confined_or_grouped(
                cpu_pool,
                cpus_range,
                util_range,
                layer_growth_weight_disable,
                load,
                util,
            )? {
                adjusted -= 1;
                trace!("{} shrunk, adjusted={}", &self.name, adjusted);
            }
        }

        if adjusted != 0 {
            trace!("{} done resizing, adjusted={}", &self.name, adjusted);
        }
        Ok(adjusted)
    }
}

struct Scheduler<'a, 'b> {
    skel: BpfSkel<'a>,
    struct_ops: Option<libbpf_rs::Link>,
    layer_specs: &'b Vec<LayerSpec>,

    sched_intv: Duration,

    cpu_pool: CpuPool,
    layers: Vec<Layer>,

    layer_preempt_weight_disable: f64,
    layer_growth_weight_disable: f64,

    proc_reader: procfs::ProcReader,
    sched_stats: Stats,

    nr_layer_cpus_ranges: Vec<(usize, usize)>,
    processing_dur: Duration,

    stats_server: StatsServer<StatsReq, StatsRes>,
}

impl<'a, 'b> Scheduler<'a, 'b> {
    fn init_layers(
        skel: &mut OpenBpfSkel,
        opts: &Opts,
        specs: &Vec<LayerSpec>,
        topo: &Topology,
    ) -> Result<()> {
        skel.maps.rodata_data.nr_layers = specs.len() as u32;
        let mut perf_set = false;

        let mut layer_iteration_order = (0..specs.len()).collect::<Vec<_>>();
        let mut layer_weights: Vec<usize> = vec![];

        for (spec_i, spec) in specs.iter().enumerate() {
            let layer = &mut skel.maps.bss_data.layers[spec_i];

            for (or_i, or) in spec.matches.iter().enumerate() {
                for (and_i, and) in or.iter().enumerate() {
                    let mt = &mut layer.matches[or_i].matches[and_i];
                    match and {
                        LayerMatch::CgroupPrefix(prefix) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_CGROUP_PREFIX as i32;
                            copy_into_cstr(&mut mt.cgroup_prefix, prefix.as_str());
                        }
                        LayerMatch::CommPrefix(prefix) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_COMM_PREFIX as i32;
                            copy_into_cstr(&mut mt.comm_prefix, prefix.as_str());
                        }
                        LayerMatch::PcommPrefix(prefix) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_PCOMM_PREFIX as i32;
                            copy_into_cstr(&mut mt.pcomm_prefix, prefix.as_str());
                        }
                        LayerMatch::NiceAbove(nice) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_NICE_ABOVE as i32;
                            mt.nice = *nice;
                        }
                        LayerMatch::NiceBelow(nice) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_NICE_BELOW as i32;
                            mt.nice = *nice;
                        }
                        LayerMatch::NiceEquals(nice) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_NICE_EQUALS as i32;
                            mt.nice = *nice;
                        }
                        LayerMatch::UIDEquals(user_id) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_USER_ID_EQUALS as i32;
                            mt.user_id = *user_id;
                        }
                        LayerMatch::GIDEquals(group_id) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_GROUP_ID_EQUALS as i32;
                            mt.group_id = *group_id;
                        }
                        LayerMatch::PIDEquals(pid) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_PID_EQUALS as i32;
                            mt.pid = *pid;
                        }
                        LayerMatch::PPIDEquals(ppid) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_PPID_EQUALS as i32;
                            mt.ppid = *ppid;
                        }
                        LayerMatch::TGIDEquals(tgid) => {
                            mt.kind = bpf_intf::layer_match_kind_MATCH_TGID_EQUALS as i32;
                            mt.tgid = *tgid;
                        }
                    }
                }
                layer.matches[or_i].nr_match_ands = or.len() as i32;
            }

            layer.nr_match_ors = spec.matches.len() as u32;

            match &spec.kind {
                LayerKind::Confined {
                    min_exec_us,
                    yield_ignore,
                    perf,
                    preempt,
                    preempt_first,
                    exclusive,
                    idle_smt,
                    growth_algo,
                    nodes,
                    slice_us,
                    weight,
                    ..
                }
                | LayerKind::Grouped {
                    min_exec_us,
                    yield_ignore,
                    perf,
                    preempt,
                    preempt_first,
                    exclusive,
                    idle_smt,
                    growth_algo,
                    nodes,
                    slice_us,
                    weight,
                    ..
                }
                | LayerKind::Open {
                    min_exec_us,
                    yield_ignore,
                    perf,
                    preempt,
                    preempt_first,
                    exclusive,
                    idle_smt,
                    growth_algo,
                    nodes,
                    slice_us,
                    weight,
                    ..
                } => {
                    layer.slice_ns = if *slice_us > 0 {
                        *slice_us * 1000
                    } else {
                        opts.slice_us * 1000
                    };
                    layer.min_exec_ns = min_exec_us * 1000;
                    layer.yield_step_ns = if *yield_ignore > 0.999 {
                        0
                    } else if *yield_ignore < 0.001 {
                        layer.slice_ns
                    } else {
                        (layer.slice_ns as f64 * (1.0 - *yield_ignore)) as u64
                    };
                    layer.preempt.write(*preempt);
                    layer.preempt_first.write(*preempt_first);
                    layer.exclusive.write(*exclusive);
                    layer.idle_smt.write(*idle_smt);
                    layer.growth_algo = growth_algo.as_bpf_enum();
                    layer.weight = if *weight <= MAX_LAYER_WEIGHT && *weight >= MIN_LAYER_WEIGHT {
                        *weight
                    } else {
                        DEFAULT_LAYER_WEIGHT
                    };
                    layer_weights.push(layer.weight.try_into().unwrap());
                    layer.perf = u32::try_from(*perf)?;
                    layer.node_mask = nodemask_from_nodes(nodes) as u64;
                    for topo_node in topo.nodes() {
                        if !nodes.contains(&topo_node.id()) {
                            continue;
                        }
                        layer.cache_mask |= cachemask_from_llcs(&topo_node.llcs()) as u64;
                    }
                }
            }

            match &spec.kind {
                LayerKind::Open { .. } | LayerKind::Grouped { .. } => {
                    layer.open.write(true);
                }
                _ => {}
            }

            perf_set |= layer.perf > 0;
        }

        layer_iteration_order.sort_by(|i, j| layer_weights[*i].cmp(&layer_weights[*j]));
        for (idx, layer_idx) in layer_iteration_order.iter().enumerate() {
            skel.maps.rodata_data.layer_iteration_order[idx] = *layer_idx as u32;
        }

        if perf_set && !compat::ksym_exists("scx_bpf_cpuperf_set")? {
            warn!("cpufreq support not available, ignoring perf configurations");
        }

        Ok(())
    }

    fn init_nodes(skel: &mut OpenBpfSkel, _opts: &Opts, topo: &Topology) {
        skel.maps.rodata_data.nr_nodes = topo.nodes().len() as u32;
        skel.maps.rodata_data.nr_llcs = 0;

        for node in topo.nodes() {
            info!(
                "configuring node {}, LLCs {:?}",
                node.id(),
                node.llcs().len()
            );
            skel.maps.rodata_data.nr_llcs += node.llcs().len() as u32;
            let raw_numa_slice = node.span().as_raw_slice();
            let node_cpumask_slice = &mut skel.maps.rodata_data.numa_cpumasks[node.id()];
            let (left, _) = node_cpumask_slice.split_at_mut(raw_numa_slice.len());
            left.clone_from_slice(raw_numa_slice);
            debug!(
                "node {} mask: {:?}",
                node.id(),
                skel.maps.rodata_data.numa_cpumasks[node.id()]
            );

            for (_, llc) in node.llcs() {
                info!("configuring llc {:?} for node {:?}", llc.id(), node.id());
                skel.maps.rodata_data.llc_numa_id_map[llc.id()] = node.id() as u32;
            }
        }

        for (_, cpu) in topo.cpus() {
            skel.maps.rodata_data.cpu_llc_id_map[cpu.id()] = cpu.llc_id() as u32;
        }
    }

    fn init(
        opts: &Opts,
        layer_specs: &'b Vec<LayerSpec>,
        open_object: &'a mut MaybeUninit<OpenObject>,
    ) -> Result<Self> {
        let nr_layers = layer_specs.len();
        let topo = Topology::new()?;
        let cpu_pool = CpuPool::new(&topo)?;

        // Open the BPF prog first for verification.
        let mut skel_builder = BpfSkelBuilder::default();
        skel_builder.obj_builder.debug(opts.verbose > 1);
        init_libbpf_logging(None);
        let mut skel = skel_builder
            .open(open_object)
            .context("failed to open BPF program")?;

        // scheduler_tick() got renamed to sched_tick() during v6.10-rc.
        let sched_tick_name = match compat::ksym_exists("sched_tick")? {
            true => "sched_tick",
            false => "scheduler_tick",
        };

        skel.progs
            .sched_tick_fentry
            .set_attach_target(0, Some(sched_tick_name.into()))
            .context("Failed to set attach target for sched_tick_fentry()")?;

        // Initialize skel according to @opts.
        // skel.struct_ops.layered_mut().exit_dump_len = opts.exit_dump_len;

        skel.maps.rodata_data.debug = opts.verbose as u32;
        skel.maps.rodata_data.slice_ns = opts.slice_us * 1000;
        skel.maps.rodata_data.max_exec_ns = if opts.max_exec_us > 0 {
            opts.max_exec_us * 1000
        } else {
            opts.slice_us * 1000 * 20
        };
        skel.maps.rodata_data.nr_possible_cpus = *NR_POSSIBLE_CPUS as u32;
        skel.maps.rodata_data.smt_enabled = cpu_pool.nr_cpus > cpu_pool.nr_cores;
        skel.maps.rodata_data.has_little_cores = topo.has_little_cores();
        skel.maps.rodata_data.disable_topology = opts.disable_topology;
        skel.maps.rodata_data.xnuma_preemption = opts.xnuma_preemption;
        skel.maps.rodata_data.dsq_iter_algo = opts.dsq_iter_algo.as_bpf_enum();
        for (cpu, sib) in cpu_pool.sibling_cpu.iter().enumerate() {
            skel.maps.rodata_data.__sibling_cpu[cpu] = *sib;
        }
        for cpu in cpu_pool.all_cpus.iter_ones() {
            skel.maps.rodata_data.all_cpus[cpu / 8] |= 1 << (cpu % 8);
        }
        Self::init_layers(&mut skel, opts, layer_specs, &topo)?;
        Self::init_nodes(&mut skel, opts, &topo);

        let mut skel = skel.load().context("Failed to load BPF program")?;

        let mut layers = vec![];
        for (idx, spec) in layer_specs.iter().enumerate() {
            layers.push(Layer::new(
                idx,
                &cpu_pool,
                &spec.name,
                spec.kind.clone(),
                &topo,
            )?);
        }
        initialize_cpu_ctxs(&skel, &topo).unwrap();

        // Other stuff.
        let proc_reader = procfs::ProcReader::new();

        // XXX If we try to refresh the cpumasks here before attaching, we
        // sometimes (non-deterministically) don't see the updated values in
        // BPF. It would be better to update the cpumasks here before we
        // attach, but the value will quickly converge anyways so it's not a
        // huge problem in the interim until we figure it out.

        // Attach.
        let stats_server = StatsServer::new(stats::server_data()).launch()?;

        let mut sched = Self {
            struct_ops: None,
            layer_specs,

            sched_intv: Duration::from_secs_f64(opts.interval),

            cpu_pool,
            layers,

            layer_preempt_weight_disable: opts.layer_preempt_weight_disable,
            layer_growth_weight_disable: opts.layer_growth_weight_disable,

            sched_stats: Stats::new(&mut skel, &proc_reader)?,

            nr_layer_cpus_ranges: vec![(0, 0); nr_layers],
            processing_dur: Default::default(),

            proc_reader,
            skel,

            stats_server,
        };

        sched
            .skel
            .attach()
            .context("Failed to attach BPF program")?;

        sched.struct_ops = Some(
            sched
                .skel
                .maps
                .layered
                .attach_struct_ops()
                .context("Failed to attach layered struct ops")?,
        );

        info!("Layered Scheduler Attached. Run `scx_layered --monitor` for metrics.");

        Ok(sched)
    }

    fn update_bpf_layer_cpumask(layer: &Layer, bpf_layer: &mut types::layer) {
        for bit in 0..layer.cpus.len() {
            if layer.cpus[bit] {
                bpf_layer.cpus[bit / 8] |= 1 << (bit % 8);
            } else {
                bpf_layer.cpus[bit / 8] &= !(1 << (bit % 8));
            }
        }
        bpf_layer.refresh_cpus = 1;
    }

    fn set_bpf_layer_preemption(layer: &mut Layer, bpf_layer: &mut types::layer, preempt: bool) {
        layer.preempt = preempt;
        bpf_layer.preempt.write(preempt);
    }

    fn refresh_cpumasks(&mut self) -> Result<()> {
        let mut updated = false;
        let num_layers = self.layers.len();

        for idx in 0..num_layers {
            match self.layers[idx].kind {
                LayerKind::Confined {
                    cpus_range,
                    util_range,
                    ..
                }
                | LayerKind::Grouped {
                    cpus_range,
                    util_range,
                    ..
                } => {
                    let load = (
                        self.sched_stats.layer_load_sums[idx],
                        self.sched_stats.total_load_sum,
                    );
                    let util = (
                        self.sched_stats.layer_utils[idx],
                        self.sched_stats.total_util,
                    );

                    // If the layer is utilizing all the adjusted load, disable
                    // preemption
                    if self.layers[idx].can_preempt && self.layer_preempt_weight_disable > 0.0 {
                        let weighted_load = load.0 / load.1;

                        if weighted_load < self.layer_preempt_weight_disable
                            && !self.layers[idx].preempt
                        {
                            trace!(
                                "enabling bpf layer preemption for {} load {:2.3} < {:2.3}",
                                &self.layers[idx].name,
                                weighted_load,
                                self.layer_preempt_weight_disable,
                            );
                            Self::set_bpf_layer_preemption(
                                &mut self.layers[idx],
                                &mut self.skel.maps.bss_data.layers[idx],
                                true,
                            );
                        }
                        if weighted_load >= self.layer_preempt_weight_disable {
                            trace!(
                                "disabling bpf layer preemption for {} load {:2.3} > {:2.3}",
                                &self.layers[idx].name,
                                weighted_load,
                                self.layer_preempt_weight_disable,
                            );
                            Self::set_bpf_layer_preemption(
                                &mut self.layers[idx],
                                &mut self.skel.maps.bss_data.layers[idx],
                                false,
                            );
                        }
                    }

                    if self.layers[idx].resize_confined_or_grouped(
                        &mut self.cpu_pool,
                        cpus_range,
                        util_range,
                        self.layer_growth_weight_disable,
                        load,
                        util,
                    )? != 0
                    {
                        Self::update_bpf_layer_cpumask(
                            &self.layers[idx],
                            &mut self.skel.maps.bss_data.layers[idx],
                        );
                        updated = true;
                    }
                }
                _ => {}
            }
        }

        if updated {
            for idx in 0..num_layers {
                let layer = &mut self.layers[idx];
                let bpf_layer = &mut self.skel.maps.bss_data.layers[idx];
                match &layer.kind {
                    LayerKind::Open { .. } => {
                        let available_cpus =
                            self.cpu_pool.available_cpus_in_mask(&layer.allowed_cpus);
                        let nr_available_cpus = available_cpus.count_ones();
                        // Open layers need the intersection of allowed
                        // cpus and available cpus.
                        layer.cpus.copy_from_bitslice(&available_cpus);
                        layer.nr_cpus = nr_available_cpus;
                        Self::update_bpf_layer_cpumask(layer, bpf_layer);
                    }
                    _ => {}
                }
            }

            self.skel.maps.bss_data.fallback_cpu = self.cpu_pool.fallback_cpu as u32;

            for (lidx, layer) in self.layers.iter().enumerate() {
                self.nr_layer_cpus_ranges[lidx] = (
                    self.nr_layer_cpus_ranges[lidx].0.min(layer.nr_cpus),
                    self.nr_layer_cpus_ranges[lidx].1.max(layer.nr_cpus),
                );
            }
        }

        Ok(())
    }

    fn step(&mut self) -> Result<()> {
        let started_at = Instant::now();
        let mut load_agg = LoadAggregator::new(self.cpu_pool.nr_cpus, false);
        self.sched_stats.refresh(
            &mut self.skel,
            &self.proc_reader,
            &mut load_agg,
            started_at,
            self.processing_dur,
        )?;
        self.refresh_cpumasks()?;
        self.processing_dur += Instant::now().duration_since(started_at);
        Ok(())
    }

    fn generate_sys_stats(
        &mut self,
        stats: &Stats,
        cpus_ranges: &mut Vec<(usize, usize)>,
    ) -> Result<SysStats> {
        let bstats = &stats.bpf_stats;
        let mut sys_stats = SysStats::new(stats, bstats, self.cpu_pool.fallback_cpu)?;

        for (lidx, (spec, layer)) in self.layer_specs.iter().zip(self.layers.iter()).enumerate() {
            let layer_stats = LayerStats::new(lidx, layer, stats, bstats, cpus_ranges[lidx]);
            sys_stats.layers.insert(spec.name.to_string(), layer_stats);
            cpus_ranges[lidx] = (layer.nr_cpus, layer.nr_cpus);
        }

        Ok(sys_stats)
    }

    fn run(&mut self, shutdown: Arc<AtomicBool>) -> Result<()> {
        let (res_ch, req_ch) = self.stats_server.channels();
        let mut next_sched_at = Instant::now() + self.sched_intv;
        let mut cpus_ranges = HashMap::<ThreadId, Vec<(usize, usize)>>::new();

        while !shutdown.load(Ordering::Relaxed)
            && !UserExitInfo::exited(&self.skel.maps.bss_data.uei)?
        {
            let now = Instant::now();

            if now >= next_sched_at {
                self.step()?;
                while next_sched_at < now {
                    next_sched_at += self.sched_intv;
                }
            }

            match req_ch.recv_deadline(next_sched_at) {
                Ok(StatsReq::Hello(tid)) => {
                    cpus_ranges.insert(
                        tid,
                        self.layers.iter().map(|l| (l.nr_cpus, l.nr_cpus)).collect(),
                    );
                    let stats = Stats::new(&mut self.skel, &self.proc_reader)?;
                    res_ch.send(StatsRes::Hello(stats))?;
                }
                Ok(StatsReq::Refresh(tid, mut stats)) => {
                    // Propagate self's layer cpu ranges into each stat's.
                    for i in 0..self.nr_layer_cpus_ranges.len() {
                        for (_, ranges) in cpus_ranges.iter_mut() {
                            ranges[i] = (
                                ranges[i].0.min(self.nr_layer_cpus_ranges[i].0),
                                ranges[i].1.max(self.nr_layer_cpus_ranges[i].1),
                            );
                        }
                        self.nr_layer_cpus_ranges[i] =
                            (self.layers[i].nr_cpus, self.layers[i].nr_cpus);
                    }

                    let mut load_agg = LoadAggregator::new(self.cpu_pool.nr_cpus, false);
                    stats.refresh(
                        &mut self.skel,
                        &self.proc_reader,
                        &mut load_agg,
                        now,
                        self.processing_dur,
                    )?;
                    let sys_stats =
                        self.generate_sys_stats(&stats, cpus_ranges.get_mut(&tid).unwrap())?;
                    res_ch.send(StatsRes::Refreshed((stats, sys_stats)))?;
                }
                Ok(StatsReq::Bye(tid)) => {
                    cpus_ranges.remove(&tid);
                    res_ch.send(StatsRes::Bye)?;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(e) => Err(e)?,
            }
        }

        self.struct_ops.take();
        UserExitInfo::read(&self.skel.maps.bss_data.uei)?.report()
    }
}

impl<'a, 'b> Drop for Scheduler<'a, 'b> {
    fn drop(&mut self) {
        if let Some(struct_ops) = self.struct_ops.take() {
            drop(struct_ops);
        }
    }
}

fn write_example_file(path: &str) -> Result<()> {
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    Ok(f.write_all(serde_json::to_string_pretty(&*EXAMPLE_CONFIG)?.as_bytes())?)
}

fn verify_layer_specs(specs: &[LayerSpec]) -> Result<()> {
    let nr_specs = specs.len();
    if nr_specs == 0 {
        bail!("No layer spec");
    }
    if nr_specs > MAX_LAYERS {
        bail!("Too many layer specs");
    }

    for (idx, spec) in specs.iter().enumerate() {
        if idx < nr_specs - 1 {
            if spec.matches.len() == 0 {
                bail!("Non-terminal spec {:?} has NULL matches", spec.name);
            }
        } else {
            if spec.matches.len() != 1 || spec.matches[0].len() != 0 {
                bail!("Terminal spec {:?} must have an empty match", spec.name);
            }
        }

        if spec.matches.len() > MAX_LAYER_MATCH_ORS {
            bail!(
                "Spec {:?} has too many ({}) OR match blocks",
                spec.name,
                spec.matches.len()
            );
        }

        for (ands_idx, ands) in spec.matches.iter().enumerate() {
            if ands.len() > NR_LAYER_MATCH_KINDS {
                bail!(
                    "Spec {:?}'s {}th OR block has too many ({}) match conditions",
                    spec.name,
                    ands_idx,
                    ands.len()
                );
            }
            for one in ands.iter() {
                match one {
                    LayerMatch::CgroupPrefix(prefix) => {
                        if prefix.len() > MAX_PATH {
                            bail!("Spec {:?} has too long a cgroup prefix", spec.name);
                        }
                    }
                    LayerMatch::CommPrefix(prefix) => {
                        if prefix.len() > MAX_COMM {
                            bail!("Spec {:?} has too long a comm prefix", spec.name);
                        }
                    }
                    LayerMatch::PcommPrefix(prefix) => {
                        if prefix.len() > MAX_COMM {
                            bail!("Spec {:?} has too long a process name prefix", spec.name);
                        }
                    }
                    _ => {}
                }
            }
        }

        match spec.kind {
            LayerKind::Confined {
                cpus_range,
                util_range,
                ..
            }
            | LayerKind::Grouped {
                cpus_range,
                util_range,
                ..
            } => {
                if let Some((cpus_min, cpus_max)) = cpus_range {
                    if cpus_min > cpus_max {
                        bail!(
                            "Spec {:?} has invalid cpus_range({}, {})",
                            spec.name,
                            cpus_min,
                            cpus_max
                        );
                    }
                }
                if util_range.0 >= util_range.1 {
                    bail!(
                        "Spec {:?} has invalid util_range ({}, {})",
                        spec.name,
                        util_range.0,
                        util_range.1
                    );
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    if opts.help_stats {
        stats::server_data().describe_meta(&mut std::io::stdout(), None)?;
        return Ok(());
    }
    // clap doesn't properly parse floats, so need to do a bounds check here, see:
    // https://github.com/clap-rs/clap/issues/4253
    if opts.layer_preempt_weight_disable < 0.0 || opts.layer_preempt_weight_disable > 1.0 {
        bail!(
            "invalid value {:2.2} for layer_preempt_weight_disable, must be 0.0..1.0",
            opts.layer_preempt_weight_disable
        );
    }
    if opts.layer_growth_weight_disable < 0.0 || opts.layer_growth_weight_disable > 1.0 {
        bail!(
            "invalid value {:2.2} for layer_growth_weight_disable, must be 0.0..1.0",
            opts.layer_growth_weight_disable
        );
    }

    let llv = match opts.verbose {
        0 => simplelog::LevelFilter::Info,
        1 => simplelog::LevelFilter::Debug,
        _ => simplelog::LevelFilter::Trace,
    };
    let mut lcfg = simplelog::ConfigBuilder::new();
    lcfg.set_time_level(simplelog::LevelFilter::Error)
        .set_location_level(simplelog::LevelFilter::Off)
        .set_target_level(simplelog::LevelFilter::Off)
        .set_thread_level(simplelog::LevelFilter::Off);
    simplelog::TermLogger::init(
        llv,
        lcfg.build(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )?;

    debug!("opts={:?}", &opts);

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();
    ctrlc::set_handler(move || {
        shutdown_clone.store(true, Ordering::Relaxed);
    })
    .context("Error setting Ctrl-C handler")?;

    if let Some(intv) = opts.monitor.or(opts.stats) {
        let shutdown_copy = shutdown.clone();
        let jh = std::thread::spawn(move || {
            stats::monitor(Duration::from_secs_f64(intv), shutdown_copy).unwrap()
        });
        if opts.monitor.is_some() {
            let _ = jh.join();
            return Ok(());
        }
    }

    if let Some(path) = &opts.example {
        write_example_file(path)?;
        return Ok(());
    }

    let mut layer_config = match opts.run_example {
        true => EXAMPLE_CONFIG.clone(),
        false => LayerConfig { specs: vec![] },
    };

    for (idx, input) in opts.specs.iter().enumerate() {
        layer_config.specs.append(
            &mut LayerSpec::parse(input)
                .context(format!("Failed to parse specs[{}] ({:?})", idx, input))?,
        );
    }

    if opts.open_metrics_format {
        warn!("open_metrics_format is deprecated");
    }

    debug!("specs={}", serde_json::to_string_pretty(&layer_config)?);
    verify_layer_specs(&layer_config.specs)?;

    // If disabling topology awareness clear out any set NUMA/LLC configs and
    // it will fallback to using all cores.
    if opts.disable_topology {
        info!("Disabling topology awareness");
        for i in 0..layer_config.specs.len() {
            let kind = &mut layer_config.specs[i].kind;
            match kind {
                LayerKind::Confined { nodes, llcs, .. }
                | LayerKind::Open { nodes, llcs, .. }
                | LayerKind::Grouped { nodes, llcs, .. } => {
                    nodes.truncate(0);
                    llcs.truncate(0);
                }
            }
        }
    }

    let mut open_object = MaybeUninit::uninit();
    let mut sched = Scheduler::init(&opts, &layer_config.specs, &mut open_object)?;
    sched.run(shutdown.clone())
}
