# The default certifi package tries to extract and use its own certificates, it doesn't work when
# the library is embedded, there is a function to get the certificates but we cannot patch the cert
# because requests only uses certifi.where to get its certificates,
#
# On linux it might be possible to have direct paths for the certificates, but on windows and macos its more
# complicated than that, so on the rust side the cert is extracted to a predetermined path and this reads from there
import rust_side  # type: ignore


def where() -> str:
    return rust_side.get_cert_path()
