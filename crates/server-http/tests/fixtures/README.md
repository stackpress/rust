These are PUBLIC TEST-ONLY credentials, generated with system LibreSSL 3.3.6.
They must never be used outside these in-memory tests. The private CA key is not
checked in. The leaf uses P-256, SAN DNS:localhost, CA:false, serverAuth and a
10-year validity from generation. Tests trust only the accompanying test CA.
The invalid-certificate proof uses a mismatching requested hostname.

Generation commands (in a disposable directory):

```sh
openssl ecparam -name prime256v1 -genkey -noout -out ca.key
openssl req -new -x509 -sha256 -key ca.key -out ca.pem -days 3650 -subj '/CN=Stackpress M0 Test CA'
openssl ecparam -name prime256v1 -genkey -noout -out server.key
openssl req -new -key server.key -out server.csr -subj '/CN=localhost'
# extensions.cnf contains CA:FALSE, digitalSignature/keyEncipherment,
# extendedKeyUsage=serverAuth, subjectAltName=DNS:localhost.
openssl x509 -req -sha256 -in server.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out server.pem -days 3650 -extfile extensions.cnf
openssl x509 -in ca.pem -outform DER -out ca.der
openssl x509 -in server.pem -outform DER -out server.der
openssl pkcs8 -topk8 -nocrypt -in server.key -outform DER -out server-key.der
```
