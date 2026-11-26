#!/usr/bin/env python3
"""Jeton relais identique à ss_crypto::jeton_relais / tenants/relais.py.
HMAC-SHA256(secret, "amane-relais/v1/" + tenant) en hexadécimal."""
import hashlib, hmac, sys

CONTEXTE = b'amane-relais/v1/'

def jeton(secret: str, tenant: str) -> str:
    return hmac.new(secret.encode('utf-8'), CONTEXTE + tenant.encode('ascii'), hashlib.sha256).hexdigest()

if __name__ == '__main__':
    secret = open('/tmp/opencode/r01/secret.txt').read().strip()
    print(jeton(secret, sys.argv[1]))