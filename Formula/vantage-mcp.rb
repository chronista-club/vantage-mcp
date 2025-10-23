class VantageMcp < Formula
  desc "Process management server for Claude Code via MCP"
  homepage "https://github.com/chronista-club/vantage-mcp"
  license "MIT OR Apache-2.0"
  head "https://github.com/chronista-club/vantage-mcp.git", branch: "main"

  # Version and download URL will be updated with each release
  version "0.1.0"
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/chronista-club/vantage-mcp/releases/download/v0.1.0/vantagemcp-macos-aarch64.tar.gz"
    sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/chronista-club/vantage-mcp/releases/download/v0.1.0/vantagemcp-macos-x86_64.tar.gz"
    sha256 "PLACEHOLDER_SHA256_MACOS_X86_64"
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/chronista-club/vantage-mcp/releases/download/v0.1.0/vantagemcp-linux-aarch64.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
  else
    url "https://github.com/chronista-club/vantage-mcp/releases/download/v0.1.0/vantagemcp-linux-x86_64.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"
  end

  depends_on "rust" => :build if build.head?

  def install
    if build.head?
      system "cargo", "install", "--root", prefix, "--path", ".", "--bin", "vantagemcp"
      bin.install "#{prefix}/bin/vantagemcp"
    else
      bin.install "vantagemcp"
    end
  end

  def caveats
    <<~EOS
      Vantage MCP Server has been installed!

      To get started:
        vantagemcp --help

      To run as MCP server:
        vantagemcp

      To enable web interface:
        vantagemcp --web
    EOS
  end

  test do
    output = shell_output("#{bin}/vantagemcp --help 2>&1", 0)
    assert_match "Vantage", output
  end
end