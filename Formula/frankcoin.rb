# Homebrew formula for the frankcoin CLI miner.
#
#   brew install maxtindall/frankcoin/frankcoin
#
# Builds from source with the Swift toolchain that ships with the Xcode
# Command Line Tools -- no full Xcode, no binary to trust, no third-party deps.
class Frankcoin < Formula
  desc "Mine franks on your own machine (proof-of-work CLI miner)"
  homepage "https://github.com/maxtindall/frankcoin"
  url "https://github.com/maxtindall/frankcoin/archive/refs/tags/v1.2.0.tar.gz"
  sha256 "2e118c011f6f482d2a45fc8a18e4bffbbd3137c792616b6fd49370e037ba7679"
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
