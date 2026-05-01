// A Dagger module to install and use product-cli in pipelines.
//
// product-cli is a knowledge graph CLI for managing features, ADRs, and
// test criteria. This module pulls prebuilt binaries from GitHub Releases
// (produced by cargo-dist) and exposes them as Files or Containers that
// other Dagger pipelines can consume.
//
// Quick examples:
//
//	# Drop the binary on disk
//	dagger -m github.com/Hafeok/product-cli call binary export --path ./product
//
//	# Run a graph health check against your repo, sandboxed
//	dagger -m github.com/Hafeok/product-cli call validate --source=.
//
//	# Get a container with `product` on PATH
//	dagger -m github.com/Hafeok/product-cli call container terminal

package main

import (
	"context"
	"fmt"

	"dagger/product/internal/dagger"
)

const releaseBaseURL = "https://github.com/Hafeok/product-cli/releases"

type Product struct{}

// Binary downloads the product CLI binary for the given platform and version
// and returns it as a File. Default version is "latest", default platform is
// linux/amd64.
func (m *Product) Binary(
	ctx context.Context,
	// +optional
	// +default="latest"
	version string,
	// +optional
	// +default="linux/amd64"
	platform dagger.Platform,
) (*dagger.File, error) {
	target, archive, binaryName, err := platformToTarget(string(platform))
	if err != nil {
		return nil, err
	}

	urlPath := "latest/download"
	if version != "" && version != "latest" {
		urlPath = "download/" + version
	}
	url := fmt.Sprintf("%s/%s/product-%s.%s", releaseBaseURL, urlPath, target, archive)

	archiveFile := dag.HTTP(url)

	extractor := dag.Container().
		From("alpine:3.20").
		WithExec([]string{"apk", "add", "--no-cache", "tar", "xz", "unzip"}).
		WithFile("/tmp/archive."+archive, archiveFile).
		WithWorkdir("/out").
		WithExec([]string{"sh", "-c", extractCommand(archive, target, binaryName)})

	return extractor.File("/out/" + binaryName), nil
}

// Container returns a minimal Debian container with the product binary on
// PATH. Use it to chain product commands into a Dagger pipeline.
func (m *Product) Container(
	ctx context.Context,
	// +optional
	// +default="latest"
	version string,
	// +optional
	platform dagger.Platform,
) (*dagger.Container, error) {
	bin, err := m.Binary(ctx, version, platform)
	if err != nil {
		return nil, err
	}

	return dag.Container(dagger.ContainerOpts{Platform: platform}).
		From("debian:bookworm-slim").
		WithFile("/usr/local/bin/product", bin, dagger.ContainerWithFileOpts{Permissions: 0o755}).
		WithEntrypoint([]string{"product"}), nil
}

// Validate runs `product graph check` against a source directory containing
// a product.toml at the root. Returns the command's stdout. Non-zero exit
// (a broken graph) becomes an error in the calling pipeline — making this
// a one-liner CI gate.
func (m *Product) Validate(
	ctx context.Context,
	// Directory containing product.toml and the docs/ tree.
	source *dagger.Directory,
	// +optional
	// +default="latest"
	version string,
) (string, error) {
	c, err := m.Container(ctx, version, "linux/amd64")
	if err != nil {
		return "", err
	}
	return c.
		WithMountedDirectory("/repo", source).
		WithWorkdir("/repo").
		WithEntrypoint([]string{}).
		WithExec([]string{"product", "graph", "check"}).
		Stdout(ctx)
}

// Context assembles an LLM context bundle for the given feature. Returns the
// bundle as a single markdown string ready to inject into an agent prompt.
func (m *Product) Context(
	ctx context.Context,
	source *dagger.Directory,
	// Feature ID, e.g. "FT-001".
	feature string,
	// +optional
	// +default=2
	depth int,
	// +optional
	// +default="latest"
	version string,
) (string, error) {
	c, err := m.Container(ctx, version, "linux/amd64")
	if err != nil {
		return "", err
	}
	return c.
		WithMountedDirectory("/repo", source).
		WithWorkdir("/repo").
		WithEntrypoint([]string{}).
		WithExec([]string{"product", "context", feature, "--depth", fmt.Sprint(depth)}).
		Stdout(ctx)
}

func platformToTarget(p string) (target, archive, binaryName string, err error) {
	if p == "" {
		p = "linux/amd64"
	}
	switch p {
	case "linux/amd64":
		return "x86_64-unknown-linux-gnu", "tar.xz", "product", nil
	case "linux/arm64":
		return "aarch64-unknown-linux-gnu", "tar.xz", "product", nil
	case "darwin/amd64":
		return "x86_64-apple-darwin", "tar.xz", "product", nil
	case "darwin/arm64":
		return "aarch64-apple-darwin", "tar.xz", "product", nil
	case "windows/amd64":
		return "x86_64-pc-windows-msvc", "zip", "product.exe", nil
	}
	return "", "", "", fmt.Errorf("unsupported platform %q (supported: linux/amd64, linux/arm64, darwin/amd64, darwin/arm64, windows/amd64)", p)
}

// extractCommand strips cargo-dist's "product-<target>/" prefix and lands the
// binary at /out/<binaryName>.
func extractCommand(archive, target, binaryName string) string {
	switch archive {
	case "tar.xz":
		return fmt.Sprintf("tar -xJf /tmp/archive.tar.xz --strip-components=1 -C /out product-%s/%s", target, binaryName)
	case "zip":
		return fmt.Sprintf("unzip -j /tmp/archive.zip 'product-%s/%s' -d /out", target, binaryName)
	}
	return "false"
}
