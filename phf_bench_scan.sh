#!/usr/bin/env bash
# phf_bench_scan.sh — 测试 rust-phf 极小表优化的基准对比脚本
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

BENCH_ARGS="--bench phf_lookup -p asm_test"

echo "════════════════════════════════════════════════════════"
echo " rust-phf 优化前后三阶段性能对比测试"
echo "════════════════════════════════════════════════════════"

# 记住当前的 commit，以便最后切回来
CURRENT_COMMIT=$(git rev-parse HEAD)

# 确保工作区是干净的，或者先 stash
if ! git diff --quiet; then
    echo "[!] 请先提交或 stash 你的本地改动再运行此脚本。"
    exit 1
fi

cleanup() {
    echo "▶ 恢复到初始分支状态..."
    git checkout "$CURRENT_COMMIT" >/dev/null 2>&1
}
trap cleanup EXIT

# ── 1. 跑当前最新版本 (Linear Scan 优化) ──────────────────
echo ""
echo "▶ [1/3] 测试阶段 3：线性扫描优化 (after_linear_scan)"
cargo bench $BENCH_ARGS -- --save-baseline after_linear_scan 2>&1 \
    | grep -E "Benchmarking|time:|change:|Performance|No change|Warming|Collecting|Finished"

# ── 2. 回滚到上一版本 (仅跳过模运算) ─────────────────────
echo ""
echo "▶ [2/3] 测试阶段 2：仅跳过模运算 (after_fast_disps)"
git checkout HEAD~1 >/dev/null 2>&1
cargo bench $BENCH_ARGS -- --save-baseline after_fast_disps 2>&1 \
    | grep -E "Benchmarking|time:|change:|Performance|No change|Warming|Collecting|Finished"

# ── 3. 回滚到最原始的版本 (完全未优化) ────────────────────
echo ""
echo "▶ [3/3] 测试阶段 1：未优化的原始 SipHash (before_all)"
git checkout HEAD~2 >/dev/null 2>&1
cargo bench $BENCH_ARGS -- --save-baseline before_all 2>&1 \
    | grep -E "Benchmarking|time:|change:|Performance|No change|Warming|Collecting|Finished"

# ── 4. 进行两两对比 ────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════"
echo " 对比结果 1：跳过模运算 vs 原始版本 (Stage 2 vs Stage 1)"
echo "════════════════════════════════════════════════════════"
git checkout "$CURRENT_COMMIT" >/dev/null 2>&1
cargo bench $BENCH_ARGS -- --baseline before_all --save-baseline after_fast_disps_vs_before 2>&1 \
    | grep -E "Benchmarking|time:|change:|Performance|No change" || true

echo ""
echo "════════════════════════════════════════════════════════"
echo " 对比结果 2：线性扫描 vs 原始版本 (Stage 3 vs Stage 1)"
echo "════════════════════════════════════════════════════════"
cargo bench $BENCH_ARGS -- --baseline before_all 2>&1 \
    | grep -E "Benchmarking|time:|change:|Performance|No change" || true

echo ""
echo "测试完成！详细 HTML 图表请见："
echo "   open target/criterion/report/index.html"
