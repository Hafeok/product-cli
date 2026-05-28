"""
TC-173: default-acknowledged-cross-cutting clears per-feature preflight gaps
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


def setup_temp_repo():
    """Create a temp product-cli repo with features and ADRs"""
    tmpdir = tempfile.mkdtemp()
    os.chdir(tmpdir)

    # Initialize product repo
    subprocess.run(["product", "init", "--name", "test-repo"], check=True)

    # Create a cross-cutting ADR
    subprocess.run(["product", "adr", "new", "Cross-cutting concern"], check=True)
    adr_id = "ADR-001"

    # Set scope to cross-cutting
    subprocess.run(["product", "adr", "scope", adr_id, "cross-cutting"], check=True)
    subprocess.run(["product", "adr", "status", adr_id, "accepted"], check=True)

    # Create two features
    subprocess.run(["product", "feature", "new", "Linked feature"], check=True)
    ft_linked = "FT-001"

    subprocess.run(["product", "feature", "new", "Unlinked feature"], check=True)
    ft_unlinked = "FT-002"

    # Link ADR to first feature
    subprocess.run(["product", "feature", "link", ft_linked, "--adr", adr_id], check=True)

    return tmpdir, adr_id, ft_linked, ft_unlinked


def test_scenario_a_baseline():
    """Scenario A: baseline without default-acknowledge"""
    tmpdir, adr_id, ft_linked, ft_unlinked = setup_temp_repo()

    try:
        # Check FT-LINKED has no gap
        result = subprocess.run(
            ["product", "preflight", ft_linked],
            capture_output=True,
            text=True
        )
        assert result.returncode == 0, f"FT-LINKED should be clean: {result.stderr}"

        # Check FT-UNLINKED has gap
        result = subprocess.run(
            ["product", "preflight", ft_unlinked],
            capture_output=True,
            text=True
        )
        assert result.returncode == 1, "FT-UNLINKED should have gap"
        assert "NOT COVERED" in result.stdout or "gap" in result.stdout.lower()
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_b_default_acknowledge_clears_gap():
    """Scenario B: default-acknowledge clears the gap"""
    tmpdir, adr_id, ft_linked, ft_unlinked = setup_temp_repo()

    try:
        # Add default-acknowledged-cross-cutting to config
        config_path = Path(tmpdir) / ".product" / "config.toml"
        update_config_toml(config_path, {
            "features": {"default-acknowledged-cross-cutting": [adr_id]}
        })

        # Check FT-LINKED still clean
        result = subprocess.run(
            ["product", "preflight", ft_linked],
            capture_output=True,
            text=True
        )
        assert result.returncode == 0, f"FT-LINKED should still be clean: {result.stderr}"

        # Check FT-UNLINKED now clean with default-acknowledged annotation
        result = subprocess.run(
            ["product", "preflight", ft_unlinked],
            capture_output=True,
            text=True
        )
        assert result.returncode == 0, f"FT-UNLINKED should be clean: {result.stderr}"
        assert "default-acknowledged" in result.stdout.lower(), \
            f"Should show default-acknowledged annotation: {result.stdout}"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_c_frontmatter_untouched():
    """Scenario C: feature frontmatter is untouched"""
    tmpdir, adr_id, ft_linked, ft_unlinked = setup_temp_repo()

    try:
        # Add default-acknowledged-cross-cutting to config
        config_path = Path(tmpdir) / ".product" / "config.toml"
        update_config_toml(config_path, {
            "features": {"default-acknowledged-cross-cutting": [adr_id]}
        })

        # Run preflight
        subprocess.run(["product", "preflight", ft_unlinked], check=True)

        # Check frontmatter doesn't contain the ADR
        ft_files = list(Path(tmpdir).glob(".product/features/FT-002-*.md"))
        assert len(ft_files) == 1
        with open(ft_files[0]) as f:
            content = f.read()
        assert adr_id not in content or "adrs-rejected:" in content, \
            "ADR should not be in adrs: list"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_d_removing_entry_restores_gap():
    """Scenario D: removing the entry restores the gap"""
    tmpdir, adr_id, ft_linked, ft_unlinked = setup_temp_repo()

    try:
        # Add then remove default-acknowledged-cross-cutting
        config_path = Path(tmpdir) / ".product" / "config.toml"
        update_config_toml(config_path, {
            "features": {"default-acknowledged-cross-cutting": [adr_id]}
        })

        # Verify it's clean
        result = subprocess.run(
            ["product", "preflight", ft_unlinked],
            capture_output=True,
            text=True
        )
        assert result.returncode == 0

        # Remove the entry
        update_config_toml(config_path, {
            "features": {"default-acknowledged-cross-cutting": []}
        })

        # Gap should return
        result = subprocess.run(
            ["product", "preflight", ft_unlinked],
            capture_output=True,
            text=True
        )
        assert result.returncode == 1, "Gap should be restored"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_e_empty_list_behaves_as_absent():
    """Scenario E: empty list behaves as absent"""
    tmpdir, adr_id, ft_linked, ft_unlinked = setup_temp_repo()

    try:
        # Add empty list
        config_path = Path(tmpdir) / ".product" / "config.toml"
        update_config_toml(config_path, {
            "features": {"default-acknowledged-cross-cutting": []}
        })

        # Should have gap (same as baseline)
        result = subprocess.run(
            ["product", "preflight", ft_unlinked],
            capture_output=True,
            text=True
        )
        assert result.returncode == 1, "Empty list should behave as absent"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


if __name__ == "__main__":
    test_scenario_a_baseline()
    test_scenario_b_default_acknowledge_clears_gap()
    test_scenario_c_frontmatter_untouched()
    test_scenario_d_removing_entry_restores_gap()
    test_scenario_e_empty_list_behaves_as_absent()
    print("All TC-173 scenarios passed!")
