class IchimiServer < Formula
  desc "Process management server for Claude Code via MCP"
  homepage "https://github.com/chronista-club/ichimi-server"
  license "MIT OR Apache-2.0"
  head "https://github.com/chronista-club/ichimi-server.git", branch: "main"

  # Version and download URL will be updated with each release
  version "0.1.0"
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/chronista-club/ichimi-server/releases/download/v0.1.0/ichimi-macos-aarch64.tar.gz"
    sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/chronista-club/ichimi-server/releases/download/v0.1.0/ichimi-macos-x86_64.tar.gz"
    sha256 "PLACEHOLDER_SHA256_MACOS_X86_64"
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/chronista-club/ichimi-server/releases/download/v0.1.0/ichimi-linux-aarch64.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
  else
    url "https://github.com/chronista-club/ichimi-server/releases/download/v0.1.0/ichimi-linux-x86_64.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"
  end

  depends_on "rust" => :build if build.head?

  def install
    if build.head?
      system "cargo", "install", "--root", prefix, "--path", ".", "--bin", "ichimi"
      bin.install "#{prefix}/bin/ichimi"
    else
      bin.install "ichimi"
    end
  end

  def caveats
    <<~EOS
      Ichimi Server has been installed!
      
      To get started:
        ichimi --help
      
      To run as MCP server:
        ichimi
      
      To enable web interface:
        ichimi --web
    EOS
  end

  test do
    output = shell_output("#{bin}/ichimi --help 2>&1", 0)
    assert_match "Ichimi Server", output
  end
end