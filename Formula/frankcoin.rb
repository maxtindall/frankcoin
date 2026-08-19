# Homebrew formula for the frankcoin CLI miner.
#
#   brew install maxtindall/frankcoin/frankcoin
#
# Builds from source with the Swift toolchain that ships with the Xcode
# Command Line Tools -- no full Xcode, no binary to trust, no third-party deps.
class Frankcoin < Formula
  desc "Mine franks on your own machine (proof-of-work CLI miner)"
  homepage "https://github.com/maxtindall/frankcoin"
  url "https://github.com/maxtindall/frankcoin/archive/refs/tags/v1.4.1.tar.gz"
  sha256 "c22adf4b4cf91a925de01096befc545ce216548e4da28452b337265139f99a3c"
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
