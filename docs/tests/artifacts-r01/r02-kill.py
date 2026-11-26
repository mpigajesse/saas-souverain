#!/usr/bin/env python3
"""R-02 — tuer la connexion TCP à 20 % du transfert d'un blob.
Envoie une requête PUT HTTP/1.1 avec Content-Length = TAILLE (500 Mo par
défaut), pousse 20 % du corps, puis coupe la connexion (RST via SO_LINGER).
"""
import socket, sys, time

HOST, PORT = '127.0.0.1', 18099
TENANT = 'cccccccc-3333-4333-8333-cccccccccccc'
KEY = 'sauvegarde-amane-1'
TAILLE = int(sys.argv[1]) if len(sys.argv) > 1 else 500 * 1024 * 1024
FRACTION = float(sys.argv[2]) if len(sys.argv) > 2 else 0.20

sys.path.insert(0, '/tmp/opencode/r01')
from jeton import jeton
secret = open('/tmp/opencode/r01/secret.txt').read().strip()
jet = jeton(secret, TENANT)

path = f'/api/blobs/{TENANT}/{KEY}'
req = (
    f'PUT {path} HTTP/1.1\r\n'
    f'Host: {HOST}:{PORT}\r\n'
    f'x-relay-token: {jet}\r\n'
    f'Content-Length: {TAILLE}\r\n'
    f'Content-Type: application/octet-stream\r\n'
    f'\r\n'
).encode()

s = socket.create_connection((HOST, PORT))
s.sendall(req)
envoye = 0
cible = int(TAILLE * FRACTION)
chunk = b'\xAB' * (256 * 1024)
t0 = time.time()
while envoye < cible:
    n = min(len(chunk), cible - envoye)
    s.sendall(chunk[:n])
    envoye += n
print(f'envoye={envoye} octets ({100*envoye/TAILLE:.1f} % de {TAILLE}) en {time.time()-t0:.2f}s')

# Rupture brutale : SO_LINGER 0 force un RST au lieu d'un FIN.
import struct
s.setsockopt(socket.SOL_SOCKET, socket.SO_LINGER, struct.pack('ii', 1, 0))
s.close()
print('connexion coupee (RST) a 20%')