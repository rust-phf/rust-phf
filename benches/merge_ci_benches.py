import os
import json
import sys

def main():
    results_dir = "./results"
    if not os.path.exists(results_dir):
        print(f"Error: results directory '{results_dir}' not found.")
        sys.exit(1)
        
    data = {}
    platforms = set()
    
    # Read all json files
    for filename in os.listdir(results_dir):
        if not filename.endswith(".json") or not filename.startswith("bench_result_"):
            continue
            
        # Filename format: bench_result_{os}_{arch}.json
        parts = filename.replace(".json", "").split("_")
        if len(parts) < 4:
            continue
        os_name = parts[2]
        arch_name = parts[3]
        platform = f"{os_name} ({arch_name})"
        platforms.add(platform)
        
        filepath = os.path.join(results_dir, filename)
        with open(filepath) as f:
            try:
                job_results = json.load(f)
            except Exception as e:
                print(f"Error loading {filepath}: {e}")
                continue
                
            for r in job_results:
                key = (r["group"], r["function"])
                if key not in data:
                    data[key] = {}
                data[key][platform] = {
                    "base": r["base_time"],
                    "new": r["new_time"],
                    "change": r["change"]
                }
                
    if not data:
        print("No benchmark results found to merge.")
        sys.exit(1)
        
    platforms = sorted(list(platforms))
    
    lines = []
    lines.append("## 📊 Cross-Platform Benchmark Results Summary")
    lines.append("")
    lines.append("| Benchmark Group | Function | Platform | Baseline | PR (Optimized) | Change % |")
    lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")
    
    for key in sorted(data.keys()):
        group, func = key
        first_row = True
        
        for platform in platforms:
            if platform not in data[key]:
                continue
                
            metrics = data[key][platform]
            base_str = f"{metrics['base']:.2f} ns"
            new_str = f"{metrics['new']:.2f} ns"
            
            change = metrics['change']
            color = "🔴" if change > 1.0 else ("🟢" if change < -1.0 else "⚪")
            change_str = f"{color} {change:+.2f}%"
            
            g_val = group if first_row else ""
            f_val = func if first_row else ""
            first_row = False
            
            lines.append(f"| {g_val} | {f_val} | {platform} | {base_str} | {new_str} | {change_str} |")
            
    markdown_table = "\n".join(lines)
    
    summary_file = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_file:
        with open(summary_file, "a") as sf:
            sf.write("\n" + markdown_table + "\n")
        print("Written report to GITHUB_STEP_SUMMARY")
    else:
        print(markdown_table)

if __name__ == "__main__":
    main()
