"""
FT-104 Integration Tests - Simplified version
"""

import json
import os
import subprocess
import tempfile
from pathlib import Path


def add_default_ack_to_config(config_path, adr_ids):
    """Add default-acknowledged-cross-cutting to config.toml"""
    import toml
    
    with open(config_path, "r") as f:
        config = toml.load(f)
    
    if "features" not in config:
        config["features"] = {}
    
    config["features"]["default-acknowledged-cross-cutting"] = adr_ids
    
    with open(config_path, "w") as f:
        toml.dump(config, f)


def test_tc173_basic():
    """TC-173: Basic test that default-acknowledge clears gaps"""
    tmpdir = tempfile.mkdtemp()
    orig_dir = os.getcwd()
    os.chdir(tmpdir)
    
    try:
        # Init repo
        subprocess.run(["product", "init", "--name", "test"], 
                      input="\n\n\n\n\n\n", text=True, check=True)
        
        # Create cross-cutting ADR
        subprocess.run(["product", "adr", "new", "CrossCutting"], check=True)
        subprocess.run(["product", "adr", "scope", "ADR-001", "cross-cutting"], check=True)
        subprocess.run(["product", "adr", "status", "ADR-001", "accepted"], check=True)
        
        # Create feature without linking
        subprocess.run(["product", "feature", "new", "TestFeature"], check=True)
        
        # Should have gap
        result = subprocess.run(["product", "preflight", "FT-001"], 
                              capture_output=True)
        assert result.returncode == 1, "Should have gap before default-ack"
        
        # Add default-acknowledged
        config_path = Path(tmpdir) / ".product" / "config.toml"
        add_default_ack_to_config(config_path, ["ADR-001"])
        
        # Should be clean now
        result = subprocess.run(["product", "preflight", "FT-001"], 
                              capture_output=True, text=True)
        assert result.returncode == 0, f"Should be clean after default-ack: {result.stderr}"
        assert "default-acknowledged" in result.stdout.lower(), \
            f"Should show annotation: {result.stdout}"
        
        print("✓ TC-173 basic test passed")
        return True
        
    finally:
        os.chdir(orig_dir)
        subprocess.run(["rm", "-rf", tmpdir], check=False)


def test_tc174_rejection():
    """TC-174: Test rejection mechanism"""
    tmpdir = tempfile.mkdtemp()
    orig_dir = os.getcwd()
    os.chdir(tmpdir)
    
    try:
        # Setup
        subprocess.run(["product", "init", "--name", "test"],
                      input="\n\n\n\n\n\n", text=True, check=True)
        subprocess.run(["product", "adr", "new", "CrossCutting"], check=True)
        subprocess.run(["product", "adr", "scope", "ADR-001", "cross-cutting"], check=True)
        subprocess.run(["product", "adr", "status", "ADR-001", "accepted"], check=True)
        subprocess.run(["product", "feature", "new", "TestFeature"], check=True)
        
        config_path = Path(tmpdir) / ".product" / "config.toml"
        add_default_ack_to_config(config_path, ["ADR-001"])
        
        # Should be clean
        result = subprocess.run(["product", "preflight", "FT-001"], capture_output=True)
        assert result.returncode == 0, "Should be clean with default-ack"
        
        # Add rejection
        subprocess.run(["product", "feature", "reject", "ADR-001",
                       "--feature", "FT-001", 
                       "--reason", "Test rejection reason"], check=True)
        
        # Should have gap again
        result = subprocess.run(["product", "preflight", "FT-001"],
                              capture_output=True, text=True)
        assert result.returncode == 1, f"Should have gap after rejection: {result.stderr}"

        # Check text output for INTENTIONAL marker
        assert "INTENTIONAL" in result.stdout.upper(), \
            f"Should show INTENTIONAL: {result.stdout}"
        assert "Test rejection reason" in result.stdout or "rejection" in result.stdout.lower(), \
            f"Should show rejection reason: {result.stdout}"
        
        print("✓ TC-174 rejection test passed")
        return True
        
    finally:
        os.chdir(orig_dir)
        subprocess.run(["rm", "-rf", tmpdir], check=False)


def test_tc175_drift():
    """TC-175: Test drift detection"""
    tmpdir = tempfile.mkdtemp()
    orig_dir = os.getcwd()
    os.chdir(tmpdir)
    
    try:
        # Setup
        subprocess.run(["product", "init", "--name", "test"],
                      input="\n\n\n\n\n\n", text=True, check=True)
        subprocess.run(["product", "adr", "new", "Alive"], check=True)
        subprocess.run(["product", "adr", "scope", "ADR-001", "cross-cutting"], check=True)
        subprocess.run(["product", "adr", "status", "ADR-001", "accepted"], check=True)
        
        subprocess.run(["product", "adr", "new", "WillBeGone"], check=True)
        subprocess.run(["product", "adr", "scope", "ADR-002", "cross-cutting"], check=True)
        subprocess.run(["product", "adr", "status", "ADR-002", "accepted"], check=True)
        
        config_path = Path(tmpdir) / ".product" / "config.toml"
        add_default_ack_to_config(config_path, ["ADR-001", "ADR-002"])
        
        # Delete ADR-002
        adr_files = list(Path(tmpdir).glob(".product/adrs/ADR-002-*.md"))
        assert len(adr_files) == 1
        os.remove(adr_files[0])
        
        # Should warn
        result = subprocess.run(["product", "graph", "check"], 
                              capture_output=True, text=True)
        assert result.returncode in [0, 2], f"Should have warnings: {result.returncode}"
        
        output = result.stderr + result.stdout
        assert "W0" in output, f"Should have warning: {output}"
        assert "ADR-002" in output or "default-acknowledged" in output.lower(), \
            f"Should mention ADR-002: {output}"
        
        print("✓ TC-175 drift test passed")
        return True
        
    finally:
        os.chdir(orig_dir)
        subprocess.run(["rm", "-rf", tmpdir], check=False)


if __name__ == "__main__":
    test_tc173_basic()
    test_tc174_rejection()
    test_tc175_drift()
    print("\n✅ All FT-104 tests passed!")
