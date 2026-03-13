class Sage < Formula
  desc "A programming language where agents are first-class citizens"
  homepage "https://github.com/cargopete/sage"
  version "0.1.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/cargopete/sage/releases/download/v0.1.1/sage-v0.1.1-aarch64-apple-darwin.tar.gz"
      sha256 "577762c6171771d2d843ae15060575a82eac247b7af32aadaad3a6fdf36afe75"
    end
  end

  depends_on "openssl@3"

  def install
    bin.install "bin/sage"
    (share/"sage/toolchain").install Dir["toolchain/*"]
  end

  def caveats
    <<~EOS
      Add this to your shell profile for fast builds:
        export SAGE_TOOLCHAIN=#{opt_share}/sage/toolchain
    EOS
  end

  test do
    (testpath/"hello.sg").write <<~EOS
      agent Main {
        on start {
          emit(42);
        }
      }
      run Main;
    EOS
    system "#{bin}/sage", "check", "hello.sg"
  end
end
