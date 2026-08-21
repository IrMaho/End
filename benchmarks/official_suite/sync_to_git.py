import os
import shutil
import sys
import subprocess

sys.stdout.reconfigure(encoding='utf-8')

SRC_WORKSPACE = r"C:\Users\ASUS\Desktop\flutter_project\endApp\1"
END_REPO = r"C:\Users\ASUS\Desktop\flutter_project\end"

print("="*65)
print("📦 TRANSFERRING BENCHMARKS & ECOSYSTEM SHOWCASE TO GIT REPO")
print("="*65)

# 1. Target Directories
target_benchmarks = os.path.join(END_REPO, "benchmarks", "official_suite")
os.makedirs(target_benchmarks, exist_ok=True)

# Copy Core Suite
src_bench = os.path.join(SRC_WORKSPACE, "benchmarks")
for item in os.listdir(src_bench):
    s_path = os.path.join(src_bench, item)
    d_path = os.path.join(target_benchmarks, item)
    if os.path.isfile(s_path):
        if not item.endswith(('.exe', '.pdb', '.obj', '.o', '.dll')):
            shutil.copy2(s_path, d_path)
            print(f"  ✔ Copied {item}")
    elif os.path.isdir(s_path):
        os.makedirs(d_path, exist_ok=True)
        for sub_item in os.listdir(s_path):
            sub_s = os.path.join(s_path, sub_item)
            sub_d = os.path.join(d_path, sub_item)
            if os.path.isfile(sub_s) and not sub_item.endswith(('.exe', '.pdb', '.obj', '.o', '.dll')):
                shutil.copy2(sub_s, sub_d)
                print(f"  ✔ Copied {item}/{sub_item}")

# 2. Copy Showcase Projects
target_examples = os.path.join(END_REPO, "examples", "ecosystem_showcase")
os.makedirs(target_examples, exist_ok=True)

src_projects = os.path.join(SRC_WORKSPACE, "projects")
if os.path.exists(src_projects):
    for proj in os.listdir(src_projects):
        p_src = os.path.join(src_projects, proj)
        p_dst = os.path.join(target_examples, proj)
        if os.path.isdir(p_src):
            if os.path.exists(p_dst):
                shutil.rmtree(p_dst)
            shutil.copytree(p_src, p_dst, ignore=shutil.ignore_patterns('*.exe', '*.pdb', '*.obj', '*.o', '*.dll'))
            print(f"  ✔ Synced project: {proj}")

# 3. Copy Frameworks
src_frameworks = os.path.join(SRC_WORKSPACE, "frameworks")
target_frameworks = os.path.join(END_REPO, "frameworks")
if os.path.exists(src_frameworks):
    os.makedirs(target_frameworks, exist_ok=True)
    for fw in os.listdir(src_frameworks):
        f_src = os.path.join(src_frameworks, fw)
        f_dst = os.path.join(target_frameworks, fw)
        if os.path.isdir(f_src):
            if os.path.exists(f_dst):
                shutil.rmtree(f_dst)
            shutil.copytree(f_src, f_dst, ignore=shutil.ignore_patterns('*.exe', '*.pdb', '*.obj', '*.o', '*.dll'))
            print(f"  ✔ Synced framework: {fw}")

print("\nFiles transferred successfully!")
