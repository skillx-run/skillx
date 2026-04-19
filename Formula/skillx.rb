class Skillx < Formula
  desc "npx for Agent Skills — fetch, scan, inject, run, clean in one command"
  homepage "https://skillx.run"
  license "Apache-2.0"
  version "0.8.2"

  on_macos do
    on_arm do
      url "https://github.com/skillx-run/skillx/releases/download/v#{version}/skillx-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
    on_intel do
      url "https://github.com/skillx-run/skillx/releases/download/v#{version}/skillx-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/skillx-run/skillx/releases/download/v#{version}/skillx-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER"
    end
    on_intel do
      url "https://github.com/skillx-run/skillx/releases/download/v#{version}/skillx-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "skillx"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/skillx --version")
  end
end
