#!/usr/bin/env bash
# phf_bench.sh — 对比 disps.len==1 快路径优化前后的 PHF 查找性能
#
# 用法：
#   chmod +x phf_bench.sh
#   ./phf_bench.sh
#
# 输出：
#   - target/criterion/  内有 HTML 报告
#   - 控制台打印 criterion 的 change% 对比

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

LIB="phf_shared/src/lib.rs"
BENCH_ARGS="--bench phf_lookup -p asm_test"

echo "════════════════════════════════════════════════════════"
echo " rust-phf  disps.len==1 fast-path  benchmark"
echo "════════════════════════════════════════════════════════"
echo ""

# ── 0. 检查当前状态 ────────────────────────────────────────
if git diff --quiet "$LIB"; then
    echo "[!] $LIB 没有未保存的修改。"
    echo "    请先应用优化（见下方 patch），或直接运行 --help 查看说明。"
    echo ""
    echo "    优化内容（phf_shared/src/lib.rs）："
    echo '    pub fn get_index(hashes: &Hashes, disps: &[(u32, u32)], len: usize) -> u32 {'
    echo '        let (d1, d2) = if disps.len() == 1 {'
    echo '            unsafe { *disps.get_unchecked(0) }'
    echo '        } else {'
    echo '            disps[(hashes.g % (disps.len() as u32)) as usize]'
    echo '        };'
    echo '        displace(hashes.f1, hashes.f2, d1, d2) % (len as u32)'
    echo '    }'
    echo ""
    read -r -p "继续（假设当前已是优化后状态）? [y/N] " ans
    [[ "$ans" =~ ^[Yy]$ ]] || exit 0
fi

# ── 1. 保存"优化后"基准 ────────────────────────────────────
echo ""
echo "▶ [1/3] 保存优化后（after）基准..."
cargo bench $BENCH_ARGS -- --save-baseline after 2>&1 \
    | grep -E "Benchmarking|time:|change:|Performance|No change|Warming|Collecting|Finished"

# ── 2. 还原原始代码，保存"优化前"基准 ─────────────────────
echo ""
echo "▶ [2/3] 还原原始代码，保存优化前（before）基准..."
git stash -- "$LIB"
cargo bench $BENCH_ARGS -- --save-baseline before 2>&1 \
    | grep -E "Benchmarking|time:|change:|Performance|No change|Warming|Collecting|Finished"

# ── 3. 恢复优化，与 before 对比 ────────────────────────────
echo ""
echo "▶ [3/3] 恢复优化，与 before 基准对比..."
git stash pop
cargo bench $BENCH_ARGS -- --baseline before 2>&1 \
    | grep -E "Benchmarking|time:|change:|Performance|No change|Warming|Collecting|Finished"

# ── 4. 结果汇总 ────────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════"
echo " 完成！详细 HTML 报告："
echo "   open target/criterion/report/index.html"
echo "════════════════════════════════════════════════════════"
