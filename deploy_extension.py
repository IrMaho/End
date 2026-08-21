import os
import shutil

root = r"c:\Users\ASUS\Desktop\flutter_project\end"
vscode_dir = os.path.join(root, "editors", "vscode")

target_dirs = [
    os.path.expanduser(r"~\.antigravity-ide\extensions\endlanguage.end-lang-0.2.0"),
    os.path.expanduser(r"~\.antigravity\extensions\endlanguage.end-lang-0.2.0"),
    os.path.expanduser(r"~\.vscode\extensions\endlanguage.end-lang-0.2.0"),
    os.path.expanduser(r"~\.cursor\extensions\endlanguage.end-lang-0.2.0"),
    os.path.expanduser(r"~\.windsurf\extensions\endlanguage.end-lang-0.2.0"),
]

files_to_copy = ["package.json", "language-configuration.json", "README.md", "icon.png"]
dirs_to_copy = ["dist", "syntaxes", "snippets"]

print("Deploying End Language Extension to Antigravity IDE & all editors...")

for target in target_dirs:
    parent = os.path.dirname(target)
    if not os.path.exists(parent):
        try:
            os.makedirs(parent, exist_ok=True)
        except Exception:
            continue
    
    if os.path.exists(target):
        shutil.rmtree(target, ignore_errors=True)
    
    os.makedirs(target, exist_ok=True)
    
    for f in files_to_copy:
        src = os.path.join(vscode_dir, f)
        if os.path.exists(src):
            shutil.copy2(src, os.path.join(target, f))
            
    for d in dirs_to_copy:
        src = os.path.join(vscode_dir, d)
        if os.path.exists(src):
            shutil.copytree(src, os.path.join(target, d), dirs_exist_ok=True)
            
    print(f"[OK] Installed into: {target}")

print("SUCCESS: Extension deployed to Antigravity IDE & all directories!")
