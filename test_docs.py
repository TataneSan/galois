import os
import re
import subprocess
import tempfile
import sys

def extract_galois_blocks(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Match blocks like ```galois ... ``` (with optional fence attributes)
    blocks = re.findall(r'```galois[^\n]*\n(.*?)\n```', content, re.DOTALL)
    return blocks

def run_galois(command, code):
    with tempfile.NamedTemporaryFile(suffix='.gal', mode='w', delete=False) as f:
        f.write(code)
        temp_name = f.name
    
    try:
        # Run cargo run -- <command> <temp_name>
        result = subprocess.run(
            ['cargo', 'run', '--quiet', '--', command, temp_name],
            capture_output=True,
            text=True
        )
        return result.returncode, result.stdout, result.stderr
    finally:
        if os.path.exists(temp_name):
            os.remove(temp_name)

def has_expected_error_marker(code):
    # Only explicit error-comment lines are treated as doctest expectations.
    for line in code.splitlines():
        stripped = line.strip()
        if stripped.startswith('// Erreur') or stripped.startswith('# Erreur') or stripped.startswith('-- Erreur'):
            return True
    return False

def should_verify_semantics(file_path, expect_error):
    if expect_error:
        return True

    normalized = file_path.replace('\\', '/')
    return normalized == 'docs/index.md' or normalized.startswith('docs/examples/')

def main():
    docs_dir = 'docs'
    all_passed = True
    total_tests = 0
    failed_tests = 0

    for root, dirs, files in os.walk(docs_dir):
        for file in files:
            if file.endswith('.md'):
                file_path = os.path.join(root, file)
                blocks = extract_galois_blocks(file_path)
                
                if not blocks:
                    continue
                
                print(f"Testing {file_path} ({len(blocks)} blocks)...")
                
                for i, block in enumerate(blocks):
                    total_tests += 1
                    # Check if the block is expected to fail
                    expect_error = has_expected_error_marker(block)
                    command = 'v' if should_verify_semantics(file_path, expect_error) else 'parser'
                    
                    return_code, stdout, stderr = run_galois(command, block)
                    
                    if expect_error:
                        if return_code == 0:
                            print(f"  Block {i} in {file_path}: Expected error but got success.")
                            print("Code:")
                            print(block)
                            all_passed = False
                            failed_tests += 1
                        else:
                            # Success (failed as expected)
                            pass
                    else:
                        if return_code != 0:
                            print(f"  Block {i} in {file_path}: Expected success but got error.")
                            print("Code:")
                            print(block)
                            print("Error:")
                            print(stderr)
                            all_passed = False
                            failed_tests += 1
                        else:
                            # Success
                            pass

    if all_passed:
        print(f"\nAll {total_tests} documentation examples passed!")
        sys.exit(0)
    else:
        print(f"\n{failed_tests}/{total_tests} documentation examples failed.")
        sys.exit(1)

if __name__ == '__main__':
    main()
