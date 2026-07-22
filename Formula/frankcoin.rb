# Homebrew formula. Builds from source, so there is no binary to trust:
#   brew install maxtindall/frankcoin/frankcoin
class Frankcoin < Formula
  desc "Mine franks on your own machine"
  homepage "https://frankcoin.website"
  url "https://github.com/maxtindall/frankcoin/archive/refs/tags/v1.1.0.tar.gz"
  license "MIT"
  head "https://github.com/maxtindall/frankcoin.git", branch: "main"

  depends_on xcode: ["15.0", :build]
  depends_on :macos

  def install
    cd "mac" do
      system "swift", "build", "-c", "release", "--disable-sandbox"
      bin.install ".build/release/frankcoin"
    end
  end

  test do
    assert_match "frankcoin", shell_output("#{bin}/frankcoin help")
  end
end
