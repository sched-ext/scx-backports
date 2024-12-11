/* automatically generated by rust-bindgen 0.70.1 */
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub const __GENTOO_NOT_FREESTANDING: u32 = 1;
pub const _STDC_PREDEF_H: u32 = 1;
pub const __STDC_IEC_559__: u32 = 1;
pub const __STDC_IEC_60559_BFP__: u32 = 201404;
pub const __STDC_IEC_559_COMPLEX__: u32 = 1;
pub const __STDC_IEC_60559_COMPLEX__: u32 = 201404;
pub const __STDC_ISO_10646__: u32 = 201706;
pub const __bool_true_false_are_defined: u32 = 1;
pub const true_: u32 = 1;
pub const false_: u32 = 0;
pub type u8_ = ::std::os::raw::c_uchar;
pub type u16_ = ::std::os::raw::c_ushort;
pub type s32 = ::std::os::raw::c_int;
pub type u32_ = ::std::os::raw::c_uint;
pub type s64 = ::std::os::raw::c_longlong;
pub type u64_ = ::std::os::raw::c_ulonglong;
pub const consts_CACHELINE_SIZE: consts = 64;
pub const consts_MAX_CPUS_SHIFT: consts = 9;
pub const consts_MAX_CPUS: consts = 512;
pub const consts_MAX_CPUS_U8: consts = 64;
pub const consts_MAX_TASKS: consts = 131072;
pub const consts_MAX_PATH: consts = 4096;
pub const consts_MAX_NUMA_NODES: consts = 64;
pub const consts_MAX_LLCS: consts = 64;
pub const consts_MAX_COMM: consts = 16;
pub const consts_MAX_LAYER_MATCH_ORS: consts = 32;
pub const consts_MAX_LAYER_NAME: consts = 64;
pub const consts_MAX_LAYERS: consts = 16;
pub const consts_MAX_LAYER_WEIGHT: consts = 10000;
pub const consts_MIN_LAYER_WEIGHT: consts = 1;
pub const consts_DEFAULT_LAYER_WEIGHT: consts = 100;
pub const consts_USAGE_HALF_LIFE: consts = 100000000;
pub const consts_RUNTIME_DECAY_FACTOR: consts = 4;
pub const consts_LAYER_LAT_DECAY_FACTOR: consts = 32;
pub const consts_DSQ_ID_SPECIAL_MASK: consts = 3221225472;
pub const consts_HI_FB_DSQ_BASE: consts = 1073741824;
pub const consts_LO_FB_DSQ_BASE: consts = 2147483648;
pub const consts_DSQ_ID_LAYER_SHIFT: consts = 16;
pub const consts_DSQ_ID_LLC_MASK: consts = 65535;
pub const consts_DSQ_ID_LAYER_MASK: consts = 1073741807;
pub const consts_MAX_CGRP_PREFIXES: consts = 32;
pub const consts_NSEC_PER_USEC: consts = 1000;
pub const consts_NSEC_PER_MSEC: consts = 1000000;
pub const consts_MSEC_PER_SEC: consts = 1000;
pub const consts_NSEC_PER_SEC: consts = 1000000000;
pub type consts = ::std::os::raw::c_uint;
pub const layer_kind_LAYER_KIND_OPEN: layer_kind = 0;
pub const layer_kind_LAYER_KIND_GROUPED: layer_kind = 1;
pub const layer_kind_LAYER_KIND_CONFINED: layer_kind = 2;
pub type layer_kind = ::std::os::raw::c_uint;
pub const layer_usage_LAYER_USAGE_OWNED: layer_usage = 0;
pub const layer_usage_LAYER_USAGE_OPEN: layer_usage = 1;
pub const layer_usage_LAYER_USAGE_SUM_UPTO: layer_usage = 1;
pub const layer_usage_NR_LAYER_USAGES: layer_usage = 2;
pub type layer_usage = ::std::os::raw::c_uint;
pub const global_stat_id_GSTAT_EXCL_IDLE: global_stat_id = 0;
pub const global_stat_id_GSTAT_EXCL_WAKEUP: global_stat_id = 1;
pub const global_stat_id_GSTAT_HI_FB_EVENTS: global_stat_id = 2;
pub const global_stat_id_GSTAT_HI_FB_USAGE: global_stat_id = 3;
pub const global_stat_id_GSTAT_LO_FB_EVENTS: global_stat_id = 4;
pub const global_stat_id_GSTAT_LO_FB_USAGE: global_stat_id = 5;
pub const global_stat_id_GSTAT_FB_CPU_USAGE: global_stat_id = 6;
pub const global_stat_id_NR_GSTATS: global_stat_id = 7;
pub type global_stat_id = ::std::os::raw::c_uint;
pub const layer_stat_id_LSTAT_SEL_LOCAL: layer_stat_id = 0;
pub const layer_stat_id_LSTAT_ENQ_WAKEUP: layer_stat_id = 1;
pub const layer_stat_id_LSTAT_ENQ_EXPIRE: layer_stat_id = 2;
pub const layer_stat_id_LSTAT_ENQ_REENQ: layer_stat_id = 3;
pub const layer_stat_id_LSTAT_MIN_EXEC: layer_stat_id = 4;
pub const layer_stat_id_LSTAT_MIN_EXEC_NS: layer_stat_id = 5;
pub const layer_stat_id_LSTAT_OPEN_IDLE: layer_stat_id = 6;
pub const layer_stat_id_LSTAT_AFFN_VIOL: layer_stat_id = 7;
pub const layer_stat_id_LSTAT_KEEP: layer_stat_id = 8;
pub const layer_stat_id_LSTAT_KEEP_FAIL_MAX_EXEC: layer_stat_id = 9;
pub const layer_stat_id_LSTAT_KEEP_FAIL_BUSY: layer_stat_id = 10;
pub const layer_stat_id_LSTAT_PREEMPT: layer_stat_id = 11;
pub const layer_stat_id_LSTAT_PREEMPT_FIRST: layer_stat_id = 12;
pub const layer_stat_id_LSTAT_PREEMPT_XLLC: layer_stat_id = 13;
pub const layer_stat_id_LSTAT_PREEMPT_XNUMA: layer_stat_id = 14;
pub const layer_stat_id_LSTAT_PREEMPT_IDLE: layer_stat_id = 15;
pub const layer_stat_id_LSTAT_PREEMPT_FAIL: layer_stat_id = 16;
pub const layer_stat_id_LSTAT_EXCL_COLLISION: layer_stat_id = 17;
pub const layer_stat_id_LSTAT_EXCL_PREEMPT: layer_stat_id = 18;
pub const layer_stat_id_LSTAT_KICK: layer_stat_id = 19;
pub const layer_stat_id_LSTAT_YIELD: layer_stat_id = 20;
pub const layer_stat_id_LSTAT_YIELD_IGNORE: layer_stat_id = 21;
pub const layer_stat_id_LSTAT_MIGRATION: layer_stat_id = 22;
pub const layer_stat_id_LSTAT_XNUMA_MIGRATION: layer_stat_id = 23;
pub const layer_stat_id_LSTAT_XLLC_MIGRATION: layer_stat_id = 24;
pub const layer_stat_id_LSTAT_XLLC_MIGRATION_SKIP: layer_stat_id = 25;
pub const layer_stat_id_LSTAT_XLAYER_WAKE: layer_stat_id = 26;
pub const layer_stat_id_LSTAT_XLAYER_REWAKE: layer_stat_id = 27;
pub const layer_stat_id_LSTAT_LLC_DRAIN_TRY: layer_stat_id = 28;
pub const layer_stat_id_LSTAT_LLC_DRAIN: layer_stat_id = 29;
pub const layer_stat_id_NR_LSTATS: layer_stat_id = 30;
pub type layer_stat_id = ::std::os::raw::c_uint;
pub const llc_layer_stat_id_LLC_LSTAT_LAT: llc_layer_stat_id = 0;
pub const llc_layer_stat_id_LLC_LSTAT_CNT: llc_layer_stat_id = 1;
pub const llc_layer_stat_id_NR_LLC_LSTATS: llc_layer_stat_id = 2;
pub type llc_layer_stat_id = ::std::os::raw::c_uint;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cpu_prox_map {
    pub cpus: [u16_; 512usize],
    pub core_end: u32_,
    pub llc_end: u32_,
    pub node_end: u32_,
    pub sys_end: u32_,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of cpu_prox_map"][::std::mem::size_of::<cpu_prox_map>() - 1040usize];
    ["Alignment of cpu_prox_map"][::std::mem::align_of::<cpu_prox_map>() - 4usize];
    ["Offset of field: cpu_prox_map::cpus"][::std::mem::offset_of!(cpu_prox_map, cpus) - 0usize];
    ["Offset of field: cpu_prox_map::core_end"]
        [::std::mem::offset_of!(cpu_prox_map, core_end) - 1024usize];
    ["Offset of field: cpu_prox_map::llc_end"]
        [::std::mem::offset_of!(cpu_prox_map, llc_end) - 1028usize];
    ["Offset of field: cpu_prox_map::node_end"]
        [::std::mem::offset_of!(cpu_prox_map, node_end) - 1032usize];
    ["Offset of field: cpu_prox_map::sys_end"]
        [::std::mem::offset_of!(cpu_prox_map, sys_end) - 1036usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct cpu_ctx {
    pub cpu: s32,
    pub current_preempt: bool,
    pub current_exclusive: bool,
    pub prev_exclusive: bool,
    pub maybe_idle: bool,
    pub yielding: bool,
    pub try_preempt_first: bool,
    pub is_big: bool,
    pub protect_owned: bool,
    pub running_owned: bool,
    pub running_fallback: bool,
    pub running_at: u64_,
    pub layer_usages: [[u64_; 2usize]; 16usize],
    pub gstats: [u64_; 7usize],
    pub lstats: [[u64_; 30usize]; 16usize],
    pub ran_current_for: u64_,
    pub usage: u64_,
    pub usage_at_idle: u64_,
    pub hi_fb_dsq_id: u64_,
    pub lo_fb_dsq_id: u64_,
    pub layer_id: u32_,
    pub task_layer_id: u32_,
    pub llc_id: u32_,
    pub node_id: u32_,
    pub perf: u32_,
    pub lo_fb_seq: u64_,
    pub lo_fb_seq_at: u64_,
    pub lo_fb_usage_base: u64_,
    pub open_preempt_layer_order: [u32_; 16usize],
    pub open_layer_order: [u32_; 16usize],
    pub prox_map: cpu_prox_map,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of cpu_ctx"][::std::mem::size_of::<cpu_ctx>() - 5432usize];
    ["Alignment of cpu_ctx"][::std::mem::align_of::<cpu_ctx>() - 8usize];
    ["Offset of field: cpu_ctx::cpu"][::std::mem::offset_of!(cpu_ctx, cpu) - 0usize];
    ["Offset of field: cpu_ctx::current_preempt"]
        [::std::mem::offset_of!(cpu_ctx, current_preempt) - 4usize];
    ["Offset of field: cpu_ctx::current_exclusive"]
        [::std::mem::offset_of!(cpu_ctx, current_exclusive) - 5usize];
    ["Offset of field: cpu_ctx::prev_exclusive"]
        [::std::mem::offset_of!(cpu_ctx, prev_exclusive) - 6usize];
    ["Offset of field: cpu_ctx::maybe_idle"][::std::mem::offset_of!(cpu_ctx, maybe_idle) - 7usize];
    ["Offset of field: cpu_ctx::yielding"][::std::mem::offset_of!(cpu_ctx, yielding) - 8usize];
    ["Offset of field: cpu_ctx::try_preempt_first"]
        [::std::mem::offset_of!(cpu_ctx, try_preempt_first) - 9usize];
    ["Offset of field: cpu_ctx::is_big"][::std::mem::offset_of!(cpu_ctx, is_big) - 10usize];
    ["Offset of field: cpu_ctx::protect_owned"]
        [::std::mem::offset_of!(cpu_ctx, protect_owned) - 11usize];
    ["Offset of field: cpu_ctx::running_owned"]
        [::std::mem::offset_of!(cpu_ctx, running_owned) - 12usize];
    ["Offset of field: cpu_ctx::running_fallback"]
        [::std::mem::offset_of!(cpu_ctx, running_fallback) - 13usize];
    ["Offset of field: cpu_ctx::running_at"][::std::mem::offset_of!(cpu_ctx, running_at) - 16usize];
    ["Offset of field: cpu_ctx::layer_usages"]
        [::std::mem::offset_of!(cpu_ctx, layer_usages) - 24usize];
    ["Offset of field: cpu_ctx::gstats"][::std::mem::offset_of!(cpu_ctx, gstats) - 280usize];
    ["Offset of field: cpu_ctx::lstats"][::std::mem::offset_of!(cpu_ctx, lstats) - 336usize];
    ["Offset of field: cpu_ctx::ran_current_for"]
        [::std::mem::offset_of!(cpu_ctx, ran_current_for) - 4176usize];
    ["Offset of field: cpu_ctx::usage"][::std::mem::offset_of!(cpu_ctx, usage) - 4184usize];
    ["Offset of field: cpu_ctx::usage_at_idle"]
        [::std::mem::offset_of!(cpu_ctx, usage_at_idle) - 4192usize];
    ["Offset of field: cpu_ctx::hi_fb_dsq_id"]
        [::std::mem::offset_of!(cpu_ctx, hi_fb_dsq_id) - 4200usize];
    ["Offset of field: cpu_ctx::lo_fb_dsq_id"]
        [::std::mem::offset_of!(cpu_ctx, lo_fb_dsq_id) - 4208usize];
    ["Offset of field: cpu_ctx::layer_id"][::std::mem::offset_of!(cpu_ctx, layer_id) - 4216usize];
    ["Offset of field: cpu_ctx::task_layer_id"]
        [::std::mem::offset_of!(cpu_ctx, task_layer_id) - 4220usize];
    ["Offset of field: cpu_ctx::llc_id"][::std::mem::offset_of!(cpu_ctx, llc_id) - 4224usize];
    ["Offset of field: cpu_ctx::node_id"][::std::mem::offset_of!(cpu_ctx, node_id) - 4228usize];
    ["Offset of field: cpu_ctx::perf"][::std::mem::offset_of!(cpu_ctx, perf) - 4232usize];
    ["Offset of field: cpu_ctx::lo_fb_seq"][::std::mem::offset_of!(cpu_ctx, lo_fb_seq) - 4240usize];
    ["Offset of field: cpu_ctx::lo_fb_seq_at"]
        [::std::mem::offset_of!(cpu_ctx, lo_fb_seq_at) - 4248usize];
    ["Offset of field: cpu_ctx::lo_fb_usage_base"]
        [::std::mem::offset_of!(cpu_ctx, lo_fb_usage_base) - 4256usize];
    ["Offset of field: cpu_ctx::open_preempt_layer_order"]
        [::std::mem::offset_of!(cpu_ctx, open_preempt_layer_order) - 4264usize];
    ["Offset of field: cpu_ctx::open_layer_order"]
        [::std::mem::offset_of!(cpu_ctx, open_layer_order) - 4328usize];
    ["Offset of field: cpu_ctx::prox_map"][::std::mem::offset_of!(cpu_ctx, prox_map) - 4392usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct llc_prox_map {
    pub llcs: [u16_; 64usize],
    pub node_end: u32_,
    pub sys_end: u32_,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of llc_prox_map"][::std::mem::size_of::<llc_prox_map>() - 136usize];
    ["Alignment of llc_prox_map"][::std::mem::align_of::<llc_prox_map>() - 4usize];
    ["Offset of field: llc_prox_map::llcs"][::std::mem::offset_of!(llc_prox_map, llcs) - 0usize];
    ["Offset of field: llc_prox_map::node_end"]
        [::std::mem::offset_of!(llc_prox_map, node_end) - 128usize];
    ["Offset of field: llc_prox_map::sys_end"]
        [::std::mem::offset_of!(llc_prox_map, sys_end) - 132usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct llc_ctx {
    pub id: u32_,
    pub cpumask: *mut bpf_cpumask,
    pub nr_cpus: u32_,
    pub vtime_now: [u64_; 16usize],
    pub queued_runtime: [u64_; 16usize],
    pub lo_fb_seq: u64_,
    pub lstats: [[u64_; 2usize]; 16usize],
    pub prox_map: llc_prox_map,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of llc_ctx"][::std::mem::size_of::<llc_ctx>() - 680usize];
    ["Alignment of llc_ctx"][::std::mem::align_of::<llc_ctx>() - 8usize];
    ["Offset of field: llc_ctx::id"][::std::mem::offset_of!(llc_ctx, id) - 0usize];
    ["Offset of field: llc_ctx::cpumask"][::std::mem::offset_of!(llc_ctx, cpumask) - 8usize];
    ["Offset of field: llc_ctx::nr_cpus"][::std::mem::offset_of!(llc_ctx, nr_cpus) - 16usize];
    ["Offset of field: llc_ctx::vtime_now"][::std::mem::offset_of!(llc_ctx, vtime_now) - 24usize];
    ["Offset of field: llc_ctx::queued_runtime"]
        [::std::mem::offset_of!(llc_ctx, queued_runtime) - 152usize];
    ["Offset of field: llc_ctx::lo_fb_seq"][::std::mem::offset_of!(llc_ctx, lo_fb_seq) - 280usize];
    ["Offset of field: llc_ctx::lstats"][::std::mem::offset_of!(llc_ctx, lstats) - 288usize];
    ["Offset of field: llc_ctx::prox_map"][::std::mem::offset_of!(llc_ctx, prox_map) - 544usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct node_ctx {
    pub id: u32_,
    pub cpumask: *mut bpf_cpumask,
    pub nr_llcs: u32_,
    pub nr_cpus: u32_,
    pub llc_mask: u64_,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of node_ctx"][::std::mem::size_of::<node_ctx>() - 32usize];
    ["Alignment of node_ctx"][::std::mem::align_of::<node_ctx>() - 8usize];
    ["Offset of field: node_ctx::id"][::std::mem::offset_of!(node_ctx, id) - 0usize];
    ["Offset of field: node_ctx::cpumask"][::std::mem::offset_of!(node_ctx, cpumask) - 8usize];
    ["Offset of field: node_ctx::nr_llcs"][::std::mem::offset_of!(node_ctx, nr_llcs) - 16usize];
    ["Offset of field: node_ctx::nr_cpus"][::std::mem::offset_of!(node_ctx, nr_cpus) - 20usize];
    ["Offset of field: node_ctx::llc_mask"][::std::mem::offset_of!(node_ctx, llc_mask) - 24usize];
};
pub const layer_match_kind_MATCH_CGROUP_PREFIX: layer_match_kind = 0;
pub const layer_match_kind_MATCH_COMM_PREFIX: layer_match_kind = 1;
pub const layer_match_kind_MATCH_PCOMM_PREFIX: layer_match_kind = 2;
pub const layer_match_kind_MATCH_NICE_ABOVE: layer_match_kind = 3;
pub const layer_match_kind_MATCH_NICE_BELOW: layer_match_kind = 4;
pub const layer_match_kind_MATCH_NICE_EQUALS: layer_match_kind = 5;
pub const layer_match_kind_MATCH_USER_ID_EQUALS: layer_match_kind = 6;
pub const layer_match_kind_MATCH_GROUP_ID_EQUALS: layer_match_kind = 7;
pub const layer_match_kind_MATCH_PID_EQUALS: layer_match_kind = 8;
pub const layer_match_kind_MATCH_PPID_EQUALS: layer_match_kind = 9;
pub const layer_match_kind_MATCH_TGID_EQUALS: layer_match_kind = 10;
pub const layer_match_kind_NR_LAYER_MATCH_KINDS: layer_match_kind = 11;
pub type layer_match_kind = ::std::os::raw::c_uint;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct layer_match {
    pub kind: ::std::os::raw::c_int,
    pub cgroup_prefix: [::std::os::raw::c_char; 4096usize],
    pub comm_prefix: [::std::os::raw::c_char; 16usize],
    pub pcomm_prefix: [::std::os::raw::c_char; 16usize],
    pub nice: ::std::os::raw::c_int,
    pub user_id: u32_,
    pub group_id: u32_,
    pub pid: u32_,
    pub ppid: u32_,
    pub tgid: u32_,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of layer_match"][::std::mem::size_of::<layer_match>() - 4156usize];
    ["Alignment of layer_match"][::std::mem::align_of::<layer_match>() - 4usize];
    ["Offset of field: layer_match::kind"][::std::mem::offset_of!(layer_match, kind) - 0usize];
    ["Offset of field: layer_match::cgroup_prefix"]
        [::std::mem::offset_of!(layer_match, cgroup_prefix) - 4usize];
    ["Offset of field: layer_match::comm_prefix"]
        [::std::mem::offset_of!(layer_match, comm_prefix) - 4100usize];
    ["Offset of field: layer_match::pcomm_prefix"]
        [::std::mem::offset_of!(layer_match, pcomm_prefix) - 4116usize];
    ["Offset of field: layer_match::nice"][::std::mem::offset_of!(layer_match, nice) - 4132usize];
    ["Offset of field: layer_match::user_id"]
        [::std::mem::offset_of!(layer_match, user_id) - 4136usize];
    ["Offset of field: layer_match::group_id"]
        [::std::mem::offset_of!(layer_match, group_id) - 4140usize];
    ["Offset of field: layer_match::pid"][::std::mem::offset_of!(layer_match, pid) - 4144usize];
    ["Offset of field: layer_match::ppid"][::std::mem::offset_of!(layer_match, ppid) - 4148usize];
    ["Offset of field: layer_match::tgid"][::std::mem::offset_of!(layer_match, tgid) - 4152usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct layer_match_ands {
    pub matches: [layer_match; 11usize],
    pub nr_match_ands: ::std::os::raw::c_int,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of layer_match_ands"][::std::mem::size_of::<layer_match_ands>() - 45720usize];
    ["Alignment of layer_match_ands"][::std::mem::align_of::<layer_match_ands>() - 4usize];
    ["Offset of field: layer_match_ands::matches"]
        [::std::mem::offset_of!(layer_match_ands, matches) - 0usize];
    ["Offset of field: layer_match_ands::nr_match_ands"]
        [::std::mem::offset_of!(layer_match_ands, nr_match_ands) - 45716usize];
};
pub const layer_growth_algo_GROWTH_ALGO_STICKY: layer_growth_algo = 0;
pub const layer_growth_algo_GROWTH_ALGO_LINEAR: layer_growth_algo = 1;
pub const layer_growth_algo_GROWTH_ALGO_REVERSE: layer_growth_algo = 2;
pub const layer_growth_algo_GROWTH_ALGO_RANDOM: layer_growth_algo = 3;
pub const layer_growth_algo_GROWTH_ALGO_TOPO: layer_growth_algo = 4;
pub const layer_growth_algo_GROWTH_ALGO_ROUND_ROBIN: layer_growth_algo = 5;
pub const layer_growth_algo_GROWTH_ALGO_BIG_LITTLE: layer_growth_algo = 6;
pub const layer_growth_algo_GROWTH_ALGO_LITTLE_BIG: layer_growth_algo = 7;
pub const layer_growth_algo_GROWTH_ALGO_RANDOM_TOPO: layer_growth_algo = 8;
pub type layer_growth_algo = ::std::os::raw::c_uint;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct layer {
    pub matches: [layer_match_ands; 32usize],
    pub nr_match_ors: ::std::os::raw::c_uint,
    pub id: ::std::os::raw::c_uint,
    pub min_exec_ns: u64_,
    pub max_exec_ns: u64_,
    pub yield_step_ns: u64_,
    pub slice_ns: u64_,
    pub fifo: bool,
    pub weight: u32_,
    pub xllc_mig_min_ns: u64_,
    pub kind: ::std::os::raw::c_int,
    pub preempt: bool,
    pub preempt_first: bool,
    pub exclusive: bool,
    pub growth_algo: ::std::os::raw::c_int,
    pub nr_tasks: u64_,
    pub cpus_seq: u64_,
    pub node_mask: u64_,
    pub llc_mask: u64_,
    pub check_no_idle: bool,
    pub perf: u32_,
    pub refresh_cpus: u64_,
    pub cpus: [u8_; 64usize],
    pub nr_cpus: u32_,
    pub nr_llc_cpus: [u32_; 64usize],
    pub llcs_to_drain: u64_,
    pub llc_drain_cnt: u32_,
    pub name: [::std::os::raw::c_char; 64usize],
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of layer"][::std::mem::size_of::<layer>() - 1463568usize];
    ["Alignment of layer"][::std::mem::align_of::<layer>() - 8usize];
    ["Offset of field: layer::matches"][::std::mem::offset_of!(layer, matches) - 0usize];
    ["Offset of field: layer::nr_match_ors"]
        [::std::mem::offset_of!(layer, nr_match_ors) - 1463040usize];
    ["Offset of field: layer::id"][::std::mem::offset_of!(layer, id) - 1463044usize];
    ["Offset of field: layer::min_exec_ns"]
        [::std::mem::offset_of!(layer, min_exec_ns) - 1463048usize];
    ["Offset of field: layer::max_exec_ns"]
        [::std::mem::offset_of!(layer, max_exec_ns) - 1463056usize];
    ["Offset of field: layer::yield_step_ns"]
        [::std::mem::offset_of!(layer, yield_step_ns) - 1463064usize];
    ["Offset of field: layer::slice_ns"][::std::mem::offset_of!(layer, slice_ns) - 1463072usize];
    ["Offset of field: layer::fifo"][::std::mem::offset_of!(layer, fifo) - 1463080usize];
    ["Offset of field: layer::weight"][::std::mem::offset_of!(layer, weight) - 1463084usize];
    ["Offset of field: layer::xllc_mig_min_ns"]
        [::std::mem::offset_of!(layer, xllc_mig_min_ns) - 1463088usize];
    ["Offset of field: layer::kind"][::std::mem::offset_of!(layer, kind) - 1463096usize];
    ["Offset of field: layer::preempt"][::std::mem::offset_of!(layer, preempt) - 1463100usize];
    ["Offset of field: layer::preempt_first"]
        [::std::mem::offset_of!(layer, preempt_first) - 1463101usize];
    ["Offset of field: layer::exclusive"][::std::mem::offset_of!(layer, exclusive) - 1463102usize];
    ["Offset of field: layer::growth_algo"]
        [::std::mem::offset_of!(layer, growth_algo) - 1463104usize];
    ["Offset of field: layer::nr_tasks"][::std::mem::offset_of!(layer, nr_tasks) - 1463112usize];
    ["Offset of field: layer::cpus_seq"][::std::mem::offset_of!(layer, cpus_seq) - 1463120usize];
    ["Offset of field: layer::node_mask"][::std::mem::offset_of!(layer, node_mask) - 1463128usize];
    ["Offset of field: layer::llc_mask"][::std::mem::offset_of!(layer, llc_mask) - 1463136usize];
    ["Offset of field: layer::check_no_idle"]
        [::std::mem::offset_of!(layer, check_no_idle) - 1463144usize];
    ["Offset of field: layer::perf"][::std::mem::offset_of!(layer, perf) - 1463148usize];
    ["Offset of field: layer::refresh_cpus"]
        [::std::mem::offset_of!(layer, refresh_cpus) - 1463152usize];
    ["Offset of field: layer::cpus"][::std::mem::offset_of!(layer, cpus) - 1463160usize];
    ["Offset of field: layer::nr_cpus"][::std::mem::offset_of!(layer, nr_cpus) - 1463224usize];
    ["Offset of field: layer::nr_llc_cpus"]
        [::std::mem::offset_of!(layer, nr_llc_cpus) - 1463228usize];
    ["Offset of field: layer::llcs_to_drain"]
        [::std::mem::offset_of!(layer, llcs_to_drain) - 1463488usize];
    ["Offset of field: layer::llc_drain_cnt"]
        [::std::mem::offset_of!(layer, llc_drain_cnt) - 1463496usize];
    ["Offset of field: layer::name"][::std::mem::offset_of!(layer, name) - 1463500usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bpf_cpumask {
    pub _address: u8,
}
