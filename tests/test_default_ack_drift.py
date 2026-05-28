"""
TC-175: graph check warns when default-acknowledged-cross-cutting drifts
"""

import os
import subprocess
import tempfile
from pathlib import Path


def setup_temp_repo_with_drift():
    """Create a temp product-cli repo with drift conditions"""
    tmpdir = tempfile.mkdtemp()
    os.chdir(tmpdir)

    # Initialize product repo
    subprocess.run(["product", "init", "--name", "test-repo"], check=True)

    # Create three cross-cutting ADRs
    for i, title in enumerate(["Alive", "Gone", "Rescoped"], start=1):
        subprocess.run(["product", "adr", "new", title], check=True)
        adr_id = f"ADR-00{i}"
        subprocess.run(["product", "adr", "scope", adr_id, "cross-cutting"], check=True)
        subprocess.run(["product", "adr", "status", adr_id, "accepted"], check=True)

    # Add all three to default-acknowledged-cross-cutting
    config_path = Path(tmpdir) / ".product" / "config.toml"
    with open(config_path, "r") as f:
        config = f.read()

    # Insert the default-acknowledged-cross-cutting line into the existing [features] section
    if "[features]" in config:
        # Add to existing [features] section
        config = config.replace(
            "[features]",
            '[features]\ndefault-acknowledged-cross-cutting = ["ADR-001", "ADR-002", "ADR-003"]'
        )
    else:
        # Fallback: append new section (shouldn't happen with modern product init)
        config += '\n[features]\ndefault-acknowledged-cross-cutting = ["ADR-001", "ADR-002", "ADR-003"]\n'

    with open(config_path, "w") as f:
        f.write(config)

    # Create a feature with rejection
    subprocess.run(["product", "feature", "new", "OptOut feature"], check=True)
    ft_optout = "FT-001"

    # Add valid rejection (ADR-001) and invalid rejection (ADR-STRAY)
    ft_files = list(Path(tmpdir).glob(".product/features/FT-001-*.md"))
    with open(ft_files[0], "a") as f:
        f.write("""
adrs-rejected:
  - id: ADR-001
    reason: "Valid rejection"
  - id: ADR-STRAY
    reason: "Invalid rejection - not in default list"
""")

    return tmpdir


def test_scenario_a_adr_no_longer_exists():
    """Scenario A: listed ADR no longer exists"""
    tmpdir = setup_temp_repo_with_drift()

    try:
        # Delete ADR-002
        adr_files = list(Path(tmpdir).glob(".product/adrs/ADR-002-*.md"))
        assert len(adr_files) == 1
        os.remove(adr_files[0])

        # Run graph check
        result = subprocess.run(
            ["product", "graph", "check"],
            capture_output=True,
            text=True,
            check=False
        )

        # Should exit 0 or 2 (warnings), not 1 (errors)
        assert result.returncode in [0, 2], \
            f"Should have warnings: exit {result.returncode}"

        output = result.stderr + result.stdout
        assert "W0" in output, "Should have warning code"
        assert "ADR-002" in output or "default-acknowledged" in output.lower(), \
            f"Should warn about missing ADR-002: {output}"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_b_adr_scope_changed():
    """Scenario B: listed ADR's scope changed away from cross-cutting"""
    tmpdir = setup_temp_repo_with_drift()

    try:
        # Change ADR-003's scope
        subprocess.run(["product", "adr", "scope", "ADR-003", "feature-specific"],
                     check=True)

        # Run graph check
        result = subprocess.run(
            ["product", "graph", "check"],
            capture_output=True,
            text=True,
            check=False
        )

        assert result.returncode in [0, 2], "Should have warnings"

        output = result.stderr + result.stdout
        assert "W0" in output, "Should have warning code"
        assert "ADR-003" in output or "scope" in output.lower(), \
            f"Should warn about scope change: {output}"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_c_feature_rejects_unlisted_adr():
    """Scenario C: feature rejects an ADR not in default-acknowledge list"""
    tmpdir = setup_temp_repo_with_drift()

    try:
        # The setup already has ADR-STRAY in the rejection list
        # Run graph check
        result = subprocess.run(
            ["product", "graph", "check"],
            capture_output=True,
            text=True,
            check=False
        )

        assert result.returncode in [0, 2], "Should have warnings"

        output = result.stderr + result.stdout
        assert "W0" in output, "Should have warning code"
        assert "ADR-STRAY" in output or "FT-001" in output, \
            f"Should warn about invalid rejection: {output}"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_d_three_warnings_coexist():
    """Scenario D: three warnings co-exist without masking each other"""
    tmpdir = setup_temp_repo_with_drift()

    try:
        # Trigger all three drift conditions:
        # 1. Delete ADR-002
        adr_files = list(Path(tmpdir).glob(".product/adrs/ADR-002-*.md"))
        os.remove(adr_files[0])

        # 2. Change ADR-003's scope
        subprocess.run(["product", "adr", "scope", "ADR-003", "domain"],
                     check=True)

        # 3. ADR-STRAY already in rejection list from setup

        # Run graph check
        result = subprocess.run(
            ["product", "graph", "check"],
            capture_output=True,
            text=True,
            check=False
        )

        assert result.returncode in [0, 2], "Should have warnings"

        output = result.stderr + result.stdout
        # Count warning indicators (each should trigger a W0XX warning)
        warning_count = output.count("W0")
        assert warning_count >= 3, \
            f"Should have at least 3 warnings, got {warning_count}: {output}"

        # Check all three conditions are mentioned
        assert "ADR-002" in output, "Should mention ADR-002"
        assert "ADR-003" in output, "Should mention ADR-003"
        assert "ADR-STRAY" in output or "FT-001" in output, "Should mention invalid rejection"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_e_fixing_warnings_independently():
    """Scenario E: fixing each warning clears it independently"""
    tmpdir = setup_temp_repo_with_drift()

    try:
        # Start with all three drift conditions
        adr_files = list(Path(tmpdir).glob(".product/adrs/ADR-002-*.md"))
        os.remove(adr_files[0])
        subprocess.run(["product", "adr", "scope", "ADR-003", "domain"], check=True)

        # Fix 1: Remove ADR-002 from config
        config_path = Path(tmpdir) / ".product" / "config.toml"
        with open(config_path, "r") as f:
            config = f.read()
        config = config.replace('"ADR-002", ', '')
        with open(config_path, "w") as f:
            f.write(config)

        result = subprocess.run(
            ["product", "graph", "check"],
            capture_output=True,
            text=True,
            check=False
        )
        output1 = result.stderr + result.stdout
        # Should not have W036 warning about missing ADR-002
        assert "W036" not in output1 or "ADR-002" not in output1, \
            f"ADR-002 drift warning should be gone: {output1}"

        # Fix 2: Remove ADR-003 from config or restore its scope
        with open(config_path, "r") as f:
            config = f.read()
        config = config.replace(', "ADR-003"', '')
        with open(config_path, "w") as f:
            f.write(config)

        result = subprocess.run(
            ["product", "graph", "check"],
            capture_output=True,
            text=True,
            check=False
        )
        output2 = result.stderr + result.stdout
        # Should not have W037 warning about scope change for ADR-003
        assert "W037" not in output2, \
            f"ADR-003 drift warning should be gone: {output2}"

        # Fix 3: Remove ADR-STRAY from feature
        ft_files = list(Path(tmpdir).glob(".product/features/FT-001-*.md"))
        with open(ft_files[0], "r") as f:
            content = f.read()
        # Remove the ADR-STRAY rejection
        lines = content.split("\n")
        filtered = []
        skip_until_next_id = False
        for line in lines:
            if "id: ADR-STRAY" in line:
                skip_until_next_id = True
                continue
            if skip_until_next_id:
                if "reason:" in line:
                    continue
                elif line.strip() and not line.startswith(" "):
                    skip_until_next_id = False
            if not skip_until_next_id:
                filtered.append(line)
        with open(ft_files[0], "w") as f:
            f.write("\n".join(filtered))

        result = subprocess.run(
            ["product", "graph", "check"],
            capture_output=True,
            text=True,
            check=False
        )
        output3 = result.stderr + result.stdout
        # Should have no FT-104 related warnings now
        # (may have other warnings from other features)
        assert "ADR-STRAY" not in output3, "ADR-STRAY warning should be gone"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


def test_scenario_f_exit_code_unchanged():
    """Scenario F: exit code unchanged by drift (warnings only)"""
    tmpdir = setup_temp_repo_with_drift()

    try:
        # Delete ADR-002 to trigger drift
        adr_files = list(Path(tmpdir).glob(".product/adrs/ADR-002-*.md"))
        os.remove(adr_files[0])

        # Run graph check
        result = subprocess.run(
            ["product", "graph", "check"],
            capture_output=True,
            text=True,
            check=False
        )

        # Exit code should be 0 (clean) or 2 (warnings), not 1 (errors)
        assert result.returncode != 1, \
            f"Drift should not cause error exit code: {result.returncode}"
    finally:
        os.chdir("/")
        subprocess.run(["rm", "-rf", tmpdir])


if __name__ == "__main__":
    test_scenario_a_adr_no_longer_exists()
    test_scenario_b_adr_scope_changed()
    test_scenario_c_feature_rejects_unlisted_adr()
    test_scenario_d_three_warnings_coexist()
    test_scenario_e_fixing_warnings_independently()
    test_scenario_f_exit_code_unchanged()
    print("All TC-175 scenarios passed!")
