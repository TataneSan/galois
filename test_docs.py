import os
import re
import subprocess
import sys
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent
DOCS_DIR = REPO_ROOT / "docs"
DOC_ARTIFACTS_DIR = REPO_ROOT / "target" / "tests-artifacts" / "docs-doctests"
CLI_DOC_PATH = REPO_ROOT / "docs" / "reference" / "cli.md"
DIAGNOSTICS_DOC_PATH = REPO_ROOT / "docs" / "reference" / "diagnostics.md"
PAQUETS_DOC_PATH = REPO_ROOT / "docs" / "reference" / "paquets.md"
CLI_SOURCE_PATH = REPO_ROOT / "src" / "main.rs"
DIAGNOSTICS_SOURCE_PATH = REPO_ROOT / "src" / "error" / "mod.rs"
MANIFEST_SOURCE_PATH = REPO_ROOT / "src" / "package" / "manifeste.rs"

CLI_COMMANDS_START = "<!-- doc-sync:cli-commands:start -->"
CLI_COMMANDS_END = "<!-- doc-sync:cli-commands:end -->"
CLI_OPTIONS_START = "<!-- doc-sync:cli-options:start -->"
CLI_OPTIONS_END = "<!-- doc-sync:cli-options:end -->"


def extract_galois_blocks(file_path):
    with open(file_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Match blocks like ```galois ... ``` (with optional fence attributes)
    return re.findall(r"```galois[^\n]*\n(.*?)\n```", content, re.DOTALL)


def create_temp_galois_file(code):
    DOC_ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)
    temp_name = DOC_ARTIFACTS_DIR / f"doc_block_{os.getpid()}_{time.time_ns()}.gal"
    temp_name.write_text(code, encoding="utf-8")
    return temp_name


def run_galois(command, code):
    temp_name = create_temp_galois_file(code)
    try:
        result = subprocess.run(
            ["cargo", "run", "--quiet", "--", command, str(temp_name)],
            capture_output=True,
            text=True,
            cwd=REPO_ROOT,
        )
        return result.returncode, result.stdout, result.stderr
    finally:
        if temp_name.exists():
            temp_name.unlink()


def has_expected_error_marker(code):
    # Only explicit error-comment lines are treated as doctest expectations.
    for line in code.splitlines():
        stripped = line.strip()
        if (
            stripped.startswith("// Erreur")
            or stripped.startswith("# Erreur")
            or stripped.startswith("-- Erreur")
        ):
            return True
    return False


def should_verify_semantics(file_path, expect_error):
    if expect_error:
        return True

    normalized = file_path.relative_to(REPO_ROOT).as_posix()
    return normalized == "docs/index.md" or normalized.startswith("docs/examples/")


def extract_marked_block(content, start_marker, end_marker):
    pattern = re.escape(start_marker) + r"(.*?)" + re.escape(end_marker)
    match = re.search(pattern, content, re.DOTALL)
    if not match:
        return None
    return match.group(1).strip()


def parse_markdown_table(table_block):
    rows = []
    for line in table_block.splitlines():
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if cells and all(re.fullmatch(r":?-{3,}:?", cell) for cell in cells):
            continue
        if cells and all(not cell for cell in cells):
            continue
        rows.append(cells)
    if len(rows) <= 1:
        return []
    return rows[1:]


def extract_inline_code_values(cell):
    values = [value.strip() for value in re.findall(r"`([^`]+)`", cell)]
    return [value for value in values if value]


def parse_scope_tokens(text):
    tokens = re.findall(r"[A-Za-zÀ-ÖØ-öø-ÿ0-9_-]+", text.lower())
    return {token for token in tokens if token not in {"et", "ou"}}


def parse_help_scope(description):
    scope_group = re.search(r"\(([^)]+)\)", description)
    if scope_group:
        return parse_scope_tokens(scope_group.group(1))
    if "globale" in description.lower():
        return {"globale"}
    return set()


def parse_help_commands_and_options(help_output):
    commands = {}
    options = {}

    lines = help_output.splitlines()
    in_commands = False
    in_options = False

    for line in lines:
        stripped = line.strip()

        if stripped == "COMMANDES:":
            in_commands = True
            in_options = False
            continue
        if stripped.startswith("OPTIONS"):
            in_commands = False
            in_options = True
            continue
        if not stripped:
            in_commands = False
            in_options = False
            continue

        if in_commands:
            match = re.match(r"^\s{2}(.+?)\s{2,}", line)
            if not match:
                continue
            signature = match.group(1).strip()
            alias_part = re.split(r"\s[<\[]", signature, maxsplit=1)[0].strip()
            names = [name.strip().lower() for name in alias_part.split(",") if name.strip()]
            if names:
                commands[names[0]] = set(names[1:])
            continue

        if in_options:
            match = re.match(r"^\s{2}(.+?)\s{2,}(.+)$", line)
            if not match:
                continue
            signature, description = match.groups()
            names = []
            for part in signature.split(","):
                token = part.strip()
                if not token:
                    continue
                names.append(token.split()[0].lower())
            if not names:
                continue
            key = next((name for name in names if name.startswith("--")), names[0])
            options[key] = {"aliases": set(names), "scope": parse_help_scope(description)}

    return commands, options


def parse_table_after_heading(content, heading):
    start = content.find(heading)
    if start == -1:
        return []

    tail = content[start + len(heading) :]
    lines = tail.splitlines()
    table_lines = []
    collecting = False

    for line in lines:
        stripped = line.strip()
        if not collecting:
            if stripped.startswith("|"):
                collecting = True
                table_lines.append(line)
            elif stripped.startswith("##"):
                break
            continue

        if stripped.startswith("|"):
            table_lines.append(line)
        elif not stripped and table_lines:
            break
        else:
            break

    if not table_lines:
        return []
    return parse_markdown_table("\n".join(table_lines))


def parse_doc_commands_from_table(content):
    block = extract_marked_block(content, CLI_COMMANDS_START, CLI_COMMANDS_END)
    if block is None:
        return None

    rows = parse_markdown_table(block)
    commands = {}
    for row in rows:
        if len(row) < 2:
            continue
        command_values = extract_inline_code_values(row[0])
        alias_values = extract_inline_code_values(row[1])
        if not command_values:
            continue
        command = command_values[0].lower()
        commands[command] = {alias.lower() for alias in alias_values}
    return commands


def parse_doc_options_from_table(content):
    block = extract_marked_block(content, CLI_OPTIONS_START, CLI_OPTIONS_END)
    if block is None:
        return None

    rows = parse_markdown_table(block)
    options = {}
    for row in rows:
        if len(row) < 2:
            continue
        option_names = [value.lower() for value in extract_inline_code_values(row[0])]
        if not option_names:
            continue
        key = next((name for name in option_names if name.startswith("--")), option_names[0])
        scope_values = extract_inline_code_values(row[1])
        if scope_values:
            scope = set()
            for value in scope_values:
                scope.update(parse_scope_tokens(value))
        else:
            scope = parse_scope_tokens(row[1])
        options[key] = {"aliases": set(option_names), "scope": scope}
    return options


def check_cli_reference_tables():
    failures = []
    cli_doc = CLI_DOC_PATH.read_text(encoding="utf-8")
    _ = CLI_SOURCE_PATH.read_text(encoding="utf-8")

    doc_commands = parse_doc_commands_from_table(cli_doc)
    if doc_commands is None:
        return [f"{CLI_DOC_PATH} ne contient pas les marqueurs {CLI_COMMANDS_START}/{CLI_COMMANDS_END}."]

    doc_options = parse_doc_options_from_table(cli_doc)
    if doc_options is None:
        return [f"{CLI_DOC_PATH} ne contient pas les marqueurs {CLI_OPTIONS_START}/{CLI_OPTIONS_END}."]

    help_run = subprocess.run(
        ["cargo", "run", "--quiet", "--", "aide"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
    )
    if help_run.returncode != 0:
        return [
            "Impossible de récupérer l'aide CLI avec `cargo run --quiet -- aide`.",
            help_run.stderr.strip(),
        ]

    cli_commands, cli_options = parse_help_commands_and_options(help_run.stdout)

    missing_commands = sorted(set(cli_commands) - set(doc_commands))
    extra_commands = sorted(set(doc_commands) - set(cli_commands))
    if missing_commands:
        failures.append(f"Commandes manquantes dans {CLI_DOC_PATH}: {', '.join(missing_commands)}")
    if extra_commands:
        failures.append(f"Commandes obsolètes dans {CLI_DOC_PATH}: {', '.join(extra_commands)}")

    for command in sorted(set(cli_commands) & set(doc_commands)):
        expected_aliases = cli_commands[command]
        documented_aliases = doc_commands[command]
        if expected_aliases != documented_aliases:
            failures.append(
                f"Aliases divergents pour `{command}` (docs={sorted(documented_aliases)}, cli={sorted(expected_aliases)})."
            )

    missing_options = sorted(set(cli_options) - set(doc_options))
    extra_options = sorted(set(doc_options) - set(cli_options))
    if missing_options:
        failures.append(f"Options manquantes dans {CLI_DOC_PATH}: {', '.join(missing_options)}")
    if extra_options:
        failures.append(f"Options obsolètes dans {CLI_DOC_PATH}: {', '.join(extra_options)}")

    for option in sorted(set(cli_options) & set(doc_options)):
        expected = cli_options[option]
        documented = doc_options[option]
        if expected["aliases"] != documented["aliases"]:
            failures.append(
                f"Aliases d'option divergents pour `{option}` (docs={sorted(documented['aliases'])}, cli={sorted(expected['aliases'])})."
            )
        if expected["scope"] != documented["scope"]:
            failures.append(
                f"Portée divergente pour `{option}` (docs={sorted(documented['scope'])}, cli={sorted(expected['scope'])})."
            )

    return failures


def check_diagnostics_code_table():
    failures = []
    source = DIAGNOSTICS_SOURCE_PATH.read_text(encoding="utf-8")
    docs = DIAGNOSTICS_DOC_PATH.read_text(encoding="utf-8")

    source_codes = set(re.findall(r'"([EW]\d{3})"', source))
    doc_codes = set(re.findall(r"^\|\s*([EW]\d{3})\s*\|", docs, flags=re.MULTILINE))

    missing = sorted(source_codes - doc_codes)
    extra = sorted(doc_codes - source_codes)
    if missing:
        failures.append(
            f"Codes absents dans {DIAGNOSTICS_DOC_PATH}: {', '.join(missing)}"
        )
    if extra:
        failures.append(
            f"Codes non implémentés documentés dans {DIAGNOSTICS_DOC_PATH}: {', '.join(extra)}"
        )

    return failures


def check_package_fields_table():
    failures = []
    source = MANIFEST_SOURCE_PATH.read_text(encoding="utf-8")
    docs = PAQUETS_DOC_PATH.read_text(encoding="utf-8")

    section_match = re.search(
        r"SectionManifeste::Package => match clé \{(.*?)\n\s*\}\s*,\n\s*SectionManifeste::Dépendances =>",
        source,
        re.DOTALL,
    )
    if not section_match:
        return ["Impossible d'extraire les champs [package] depuis src/package/manifeste.rs."]

    source_fields = {
        field.lower() for field in re.findall(r'"([^"]+)"\s*=>', section_match.group(1))
    }
    source_required = {
        field.lower() for field in re.findall(r"Champ obligatoire '([^']+)' manquant", source)
    }

    rows = parse_table_after_heading(docs, "### `[package]`")
    if not rows:
        return [f"Impossible de trouver le tableau `[package]` dans {PAQUETS_DOC_PATH}."]

    doc_fields = set()
    doc_required = set()
    for row in rows:
        if len(row) < 3:
            continue
        field_name = re.sub(r"`", "", row[0]).strip().lower()
        if not field_name:
            continue
        doc_fields.add(field_name)
        if row[2].strip().lower() == "oui":
            doc_required.add(field_name)

    missing_fields = sorted(source_fields - doc_fields)
    extra_fields = sorted(doc_fields - source_fields)
    if missing_fields:
        failures.append(
            f"Champs [package] absents dans {PAQUETS_DOC_PATH}: {', '.join(missing_fields)}"
        )
    if extra_fields:
        failures.append(
            f"Champs [package] non implémentés documentés dans {PAQUETS_DOC_PATH}: {', '.join(extra_fields)}"
        )

    if source_required != doc_required:
        failures.append(
            f"Champs [package] requis divergents (docs={sorted(doc_required)}, code={sorted(source_required)})."
        )

    return failures


def run_consistency_checks():
    checks = [
        ("Référence CLI (commandes/options)", check_cli_reference_tables),
        ("Table des codes diagnostics", check_diagnostics_code_table),
        ("Table des champs [package]", check_package_fields_table),
    ]
    results = []

    print("\nRunning documentation consistency checks...")
    for check_name, check in checks:
        failures = check()
        if failures:
            print(f"  ✗ {check_name}")
            for failure in failures:
                print(f"    - {failure}")
        else:
            print(f"  ✓ {check_name}")
        results.append((check_name, failures))

    return results


def main():
    all_passed = True
    total_tests = 0
    failed_tests = 0

    for root, _, files in os.walk(DOCS_DIR):
        for file in files:
            if not file.endswith(".md"):
                continue

            file_path = Path(root) / file
            blocks = extract_galois_blocks(file_path)
            if not blocks:
                continue

            display_path = file_path.relative_to(REPO_ROOT).as_posix()
            print(f"Testing {display_path} ({len(blocks)} blocks)...")

            for i, block in enumerate(blocks):
                total_tests += 1
                expect_error = has_expected_error_marker(block)
                command = "v" if should_verify_semantics(file_path, expect_error) else "parser"

                return_code, _, stderr = run_galois(command, block)

                if expect_error:
                    if return_code == 0:
                        print(f"  Block {i} in {display_path}: Expected error but got success.")
                        print("Code:")
                        print(block)
                        all_passed = False
                        failed_tests += 1
                else:
                    if return_code != 0:
                        print(f"  Block {i} in {display_path}: Expected success but got error.")
                        print("Code:")
                        print(block)
                        print("Error:")
                        print(stderr)
                        all_passed = False
                        failed_tests += 1

    consistency_results = run_consistency_checks()
    total_tests += len(consistency_results)
    for _, failures in consistency_results:
        if failures:
            all_passed = False
            failed_tests += 1

    if all_passed:
        print(f"\nAll {total_tests} documentation checks passed!")
        sys.exit(0)
    else:
        print(f"\n{failed_tests}/{total_tests} documentation checks failed.")
        sys.exit(1)


if __name__ == "__main__":
    main()
