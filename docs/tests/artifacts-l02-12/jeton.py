#!/usr/bin/env python3
"""Jeton relais identique à ss_crypto::jeton_relais / tenants/relais.py.
HMAC-SHA256(secret, "amane-relais/v1/" + tenant) en hexadécimal.

Usage : jeton.py <secret_hex_ou_texte> <tenant_uuid>
"""
import hashlib, hmac, sys

CONTEXTE = b'amane-relais/v1/'

def jeton(secret: str, tenant: str) -> str:
    return hmac.new(secret.encode('utf-8'), CONTEXTE + tenant.encode('ascii'), hashlib.sha256).hexdigest()

if __name__ == '__main__':
    print(jeton(sys.argv[1], sys.argv[2]))