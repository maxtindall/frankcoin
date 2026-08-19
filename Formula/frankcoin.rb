# Homebrew formula for the frankcoin CLI miner.
#
#   brew install maxtindall/frankcoin/frankcoin
#
# Builds from source with the Swift toolchain that ships with the Xcode
# Command Line Tools -- no full Xcode, no binary to trust, no third-party deps.
class Frankcoin < Formula
  desc "Mine franks on your own machine (proof-of-work CLI miner)"
  homepage "https://github.com/maxtindall/frankcoin"
  url "https://github.com/maxtindall/frankcoin/archive/refs/tags/v1.4.0.tar.gz"
  sha256 "effd96eb5e45dc319a3e2d5672e0d55a4cc04221be7eaa861936435ae731f246"
  license "MIT"
  head "https://github.com/maxtindall/frankcoin.git", branch: "main"

  depends_on :macos

  def install
    cd "mac" do
      system "swift", "build", "-c", "release", "--disable-sandbox"
      bin.install ".build/release/frankcoin"
    end
  end

  test do
    assert_match "mine franks on your own machine", shell_output("#{bin}/frankcoin help")
  end
end
