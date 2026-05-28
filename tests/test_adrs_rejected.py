"""
TC-174: adrs-rejected re-introduces the gap with severity intentional
"""

import json
import os
import subprocess
import tempfile
import tomllib
from pathlib import Path


def update_config_toml(config_path, updates):
    """Update config.toml by appending to existing sections"""
    with open(config_path, "r") as f:
        lines = f.readlines()

    # For each section we want to update
    for section, values in updates.items():
        section_header = f"[{section}]"
        section_found = False
        insert_index = None

        # Find the section
        for i, line in enumerate(lines):
            if line.strip() == section_header:
                section_found = True
                # Find the end of this section (next section or EOF)
                insert_index = i + 1
                for j in range(i + 1, len(lines)):
                    if lines[j].strip().startswith("[") and lines[j].strip().endswith("]"):
                        insert_index = j
                        break
                else:
                    insert_index = len(lines)
                break

        # Format the values to add
        new_lines = []
        for key, value in values.items():
            if isinstance(value, list):
                new_lines.append(f'{key} = {json.dumps(value)}\n')
            elif isinstance(value, str):
                new_lines.append(f'{key} = "{value}"\n')
            elif isinstance(value, bool):
                new_lines.append(f'{key} = {str(value).lower()}\n')
            elif isinstance(value, (int, float)):
                new_lines.append(f'{key} = {value}\n')

        if section_found and insert_index is not None:
            # Insert before the next section (or at end)
            lines[insert_index:insert_index] = new_lines
        else:
            # Section doesn't exist, add it at the end
            lines.append(f"\n{section_header}\n")
            lines.extend(new_lines)

    # Write back
    with open(config_path, "w") as f:
        f.writelines(lines)


def setup_temp_repo_with_default_ack():
    """Create a temp product-cli repo with default-acknowledged ADR"""
    tmpdir = tempfile.mkdtemp()
    os.chdir(tmpdir)

    # Initialize product repo
    subprocess.run(["product", "init", "--name", "test-repo"], check=True)

    # Create a cross-cutting ADR
    subprocess.run(["product", "adr", "new", "Cross-cutting concern"], check=True)
    adr_id = "ADR-001"
    subprocess.run(["product", "adr", "scope", adr_id, "cross-cutting"], check=True)
    subprocess.run(["product", "adr", "status", adr_id, "accepted"], check=True)

    # Add default-acknowledged-cross-cutting to config
    config_path = Path(tmpdir) / ".product" / "config.toml"
    update_config_toml(config_path, {
        "features": {"default-acknowledged-cross-cutting": [adr_id]}
    })

    # Create a feature
    subprocess.run(["product", "feature", "new", "OptOut feature"], check=True)
    ft_optout = "FT-001"

    return tmpdir, adr_id, ft_optout


def test_scenario_a_rejection_surfaces_as_distinct_gap():
    """Scenario A: rejection surfaces as a distinct gap kind"""
    tmpdir, adr_id, ft_optout = setup_temp_repo_with_default_ack()

    try:
        # Add rejection to feature frontmatter
        subprocess.run([
            "product", "feature", "reject", adr_id,
            "--feature", ft_optout,
            "--reason", "This feature uses an alternative pattern because of test reasons."
        ], check=True)

        # Check JSON output
        result = subprocess.run(
            ["product", "preflight", ft_optout, "--format", "json"],
            capture_output=True,
            text=True,
            check=False
        )

        # Should fail (exit 1) because rejection counts as a gap
        assert result.returncode == 1, f"Should have gap: {result.stderr}"

        output = json.loads(result.stdout)
        # Find the ADR in cross_cutting_gaps
        adr_gap = next((g for g in output.get("cross_cutting_gaps", [])
                       if g["adr_id"] == adr_id), None)
        assert adr_gap is not None, "Should have gap for rejected ADR"
        assert adr_gap["status"] == "intentional", \
            f"Status should be intentional, got {adr_gap['status']}"
        assert "test reasons" in adr_gap.get("reason", ""), \
            "Should include rejection reason"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_b_text_format_renders_rejection():
    """Scenario B: text format renders the rejection visibly"""
    tmpdir, adr_id, ft_optout = setup_temp_repo_with_default_ack()

    try:
        # Add rejection
        subprocess.run([
            "product", "feature", "reject", adr_id,
            "--feature", ft_optout,
            "--reason", "Alternative pattern used"
        ], check=True)

        # Check text output
        result = subprocess.run(
            ["product", "preflight", ft_optout],
            capture_output=True,
            text=True,
            check=False
        )

        assert "INTENTIONAL" in result.stdout.upper(), \
            f"Should show INTENTIONAL: {result.stdout}"
        assert "Alternative pattern" in result.stdout or "reason" in result.stdout.lower(), \
            "Should show rejection reason snippet"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_c_empty_reason_rejected():
    """Scenario C: empty reason is rejected at parse time"""
    tmpdir, adr_id, ft_optout = setup_temp_repo_with_default_ack()

    try:
        # Try to add rejection with empty reason
        result = subprocess.run([
            "product", "feature", "reject", adr_id,
            "--feature", ft_optout,
            "--reason", ""
        ], capture_output=True, text=True, check=False)

        # Should fail
        assert result.returncode != 0, "Empty reason should be rejected"
        assert "reason" in result.stderr.lower() or "empty" in result.stderr.lower(), \
            f"Error should mention reason requirement: {result.stderr}"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_d_rejection_without_default_ack():
    """Scenario D: rejection without default-acknowledge is incoherent"""
    tmpdir = tempfile.mkdtemp()
    os.chdir(tmpdir)

    try:
        # Initialize without default-acknowledged-cross-cutting
        subprocess.run(["product", "init", "--name", "test-repo"], check=True)

        # Create ADR and feature
        subprocess.run(["product", "adr", "new", "Cross-cutting"], check=True)
        adr_id = "ADR-001"
        subprocess.run(["product", "adr", "scope", adr_id, "cross-cutting"], check=True)
        subprocess.run(["product", "adr", "status", adr_id, "accepted"], check=True)

        subprocess.run(["product", "feature", "new", "Test feature"], check=True)
        ft_optout = "FT-001"

        # Add rejection even though ADR is not default-acknowledged
        subprocess.run([
            "product", "feature", "reject", adr_id,
            "--feature", ft_optout,
            "--reason", "Test rejection"
        ], check=True)

        # Run graph check - should warn
        result = subprocess.run(
            ["product", "graph", "check"],
            capture_output=True,
            text=True,
            check=False
        )

        # Should exit 0 (warnings only), not 1 (errors)
        assert result.returncode == 0 or result.returncode == 2, \
            f"Should have warnings, not errors: exit {result.returncode}"

        # Check for warning about the incoherent rejection
        output = result.stderr + result.stdout
        assert "W0" in output and (ft_optout in output or adr_id in output), \
            f"Should warn about incoherent rejection: {output}"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_e_feature_reject_verb():
    """Scenario E: product feature reject verb wires the frontmatter"""
    tmpdir, adr_id, ft_optout = setup_temp_repo_with_default_ack()

    try:
        # Use the reject verb
        result = subprocess.run([
            "product", "feature", "reject", adr_id,
            "--feature", ft_optout,
            "--reason", "Stated rationale here."
        ], capture_output=True, text=True, check=True)

        # Verify frontmatter was updated
        ft_files = list(Path(tmpdir).glob(".product/features/FT-001-*.md"))
        assert len(ft_files) == 1
        with open(ft_files[0]) as f:
            content = f.read()
        assert "adrs-rejected:" in content, "Should have adrs-rejected field"
        assert adr_id in content
        assert "Stated rationale here." in content

        # Re-running should be idempotent
        result2 = subprocess.run([
            "product", "feature", "reject", adr_id,
            "--feature", ft_optout,
            "--reason", "Updated rationale."
        ], capture_output=True, text=True, check=True)

        with open(ft_files[0]) as f:
            content2 = f.read()
        assert content2.count("adrs-rejected:") == 1, "Should not duplicate field"
        assert "Updated rationale." in content2, "Should update reason"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


if __name__ == "__main__":
    test_scenario_a_rejection_surfaces_as_distinct_gap()
    test_scenario_b_text_format_renders_rejection()
    test_scenario_c_empty_reason_rejected()
    test_scenario_d_rejection_without_default_ack()
    test_scenario_e_feature_reject_verb()
    print("All TC-174 scenarios passed!")
