# copy this for other OSes
# https://go.dev/src/crypto/x509/


# The default certifi package tries to extract and use its own certificates, it doesn't work when
# the library is embedded, there is a function to get the certificates but we cannot patch the cert
# because requests only uses certifi.where to get its certificates, this uses system certificates instead
# like how some linux distros does it


def where() -> str:
    return "/etc/ssl/certs/ca-certificates.crt"
