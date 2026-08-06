import subprocess
import os
import sys
import json

def run_cmd(args, cwd=None):
    print(f"Running: {' '.join(args)} in {cwd or os.getcwd()}")
    res = subprocess.run(args, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if res.returncode != 0:
        print(f"Error running {' '.join(args)}:")
        print(res.stdout)
        print(res.stderr)
        sys.exit(res.returncode)
    return res.stdout

def main():
    bench_args = ["cargo", "bench", "--bench", "phf_lookup", "-p", "asm_test", "--"]
    
    # 1. Fetch origin/main to ensure we have the base branch commit
    run_cmd(["git", "fetch", "origin", "main"])
    
    # 2. Run PR benchmarks (Stage 3)
    print("Running PR benchmarks...")
    run_cmd(bench_args + ["--save-baseline", "pr"])
    
    # 3. Revert phf/phf_shared to origin/main (Base)
    print("Reverting phf and phf_shared to origin/main...")
    run_cmd(["git", "checkout", "origin/main", "--", "phf", "phf_shared"])
    
    # 4. Run base benchmarks (Stage 1)
    print("Running Base benchmarks...")
    # Clean targets to force compilation with base code
    run_cmd(["cargo", "clean", "-p", "phf"])
    run_cmd(["cargo", "clean", "-p", "phf_shared"])
    run_cmd(["cargo", "clean", "-p", "asm_test"])
    run_cmd(bench_args + ["--save-baseline", "base"])
    
    # 5. Restore PR libraries
    print("Restoring PR libraries...")
    run_cmd(["git", "checkout", "HEAD", "--", "phf", "phf_shared"])
    run_cmd(["git", "reset", "HEAD", "phf", "phf_shared"])
    
    # 6. Run comparison
    print("Running comparison benchmarks...")
    run_cmd(["cargo", "clean", "-p", "phf"])
    run_cmd(["cargo", "clean", "-p", "phf_shared"])
    run_cmd(["cargo", "clean", "-p", "asm_test"])
    run_cmd(bench_args + ["--baseline", "base"])
    
    # 7. Generate JSON report
    generate_json_report()

def generate_json_report():
    criterion_dir = "target/criterion"
    if not os.path.exists(criterion_dir):
        print("No criterion directory found.")
        return

    results = []
    groups = [d for d in os.listdir(criterion_dir) if os.path.isdir(os.path.join(criterion_dir, d)) and d != "report"]
    
    for group in sorted(groups):
        group_path = os.path.join(criterion_dir, group)
        functions = [d for d in os.listdir(group_path) if os.path.isdir(os.path.join(group_path, d))]
        
        for func in sorted(functions):
            func_path = os.path.join(group_path, func)
            
            new_est_path = os.path.join(func_path, "new", "estimates.json")
            base_est_path = os.path.join(func_path, "base", "estimates.json")
            
            if not os.path.exists(new_est_path) or not os.path.exists(base_est_path):
                continue
                
            with open(new_est_path) as f:
                new_data = json.load(f)
            new_time = new_data["slope"]["point_estimate"]
            
            with open(base_est_path) as f:
                base_data = json.load(f)
            base_time = base_data["slope"]["point_estimate"]
            
            change = (new_time - base_time) / base_time * 100
            
            results.append({
                "group": group,
                "function": func,
                "base_time": base_time,
                "new_time": new_time,
                "change": change
            })
            
    with open("bench_result.json", "w") as f:
        json.dump(results, f, indent=2)
    print("Saved results to bench_result.json")

if __name__ == "__main__":
    main()
