class EndLang < Formula
  desc "The AI-First Zero-GC Systems Programming Language"
  homepage "https://github.com/IrMaho/End"
  version "0.4.0-alpha"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/IrMaho/End/releases/download/v0.4.0-alpha/end-v0.4.0-alpha-macos-arm64.tar.gz"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/IrMaho/End/releases/download/v0.4.0-alpha/end-v0.4.0-alpha-macos-x64.tar.gz"
  else
    url "https://github.com/IrMaho/End/releases/download/v0.4.0-alpha/end-v0.4.0-alpha-linux-x64.tar.gz"
  end

  def install
    bin.install "bin/endc"
    bin.install "bin/end"
    prefix.install "std"
    prefix.install "Architecture.toml"
  end

  test do
    system "#{bin}/end", "eval", "15 * 4 + 20"
  end
end
