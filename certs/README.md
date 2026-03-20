# 🔐 antinna.in — Cloudflare + Salvo TLS Setup (Full Strict)

This guide explains how to securely configure:

- Cloudflare Origin Certificate (15 years)
- Rust + Salvo HTTPS server
- Cloudflare SSL mode = Full (strict)

Domain used in this guide:
antinna.in

```bash
# 🏗 Architecture Overview

Client (HTTPS / HTTP3)
        ↓
Cloudflare Edge (Public TLS)
        ↓
Encrypted HTTPS (validated origin cert)
        ↓
Rust Salvo Server (Origin)

Cloudflare handles public TLS.
Origin certificate is only used between Cloudflare and your server.
```

# 1️⃣ Generate Cloudflare Origin Certificate

In Cloudflare Dashboard:

1. Select domain: antinna.in
2. Go to SSL/TLS → Origin Server
3. Click "Create Certificate"
4. Select:
   - RSA (default is fine)
   - 15 years
5. Add hostnames:
   - antinna.in
   - *.antinna.in (optional but recommended)

Click Create.

You will receive:

- Origin Certificate (PEM format)
- Private Key (PEM format)

# 2️⃣ Project File Structure

Place certificate files inside your project:
```
backend/
│
├── certs/
│   ├── origin-cert.pem
│   └── origin-key.pem
│
├── src/
│   └── main.rs
│
└── Cargo.toml
```
⚠ IMPORTANT

- Never commit private keys to Git <i can but in private, sorry>
- Add this to .gitignore:

`certs/*`


# 4️⃣ Set Cloudflare SSL Mode (CRITICAL)

In Cloudflare Dashboard:

SSL/TLS → Overview → Select:

Full (strict)

Do NOT use:
- Flexible ❌
- Full (non-strict) ❌

Full (strict) ensures Cloudflare validates the origin certificate.


# 5️⃣ Enable Proxy (Orange Cloud)

Go to DNS settings:

Make sure your A record for antinna.in is:

Type: A
Name: antinna.in
Content: YOUR_SERVER_IP
Proxy Status: Proxied (Orange Cloud ON)

If proxy is OFF (grey cloud), this setup will not behave correctly.


# 6️⃣ Enable HTTP/3 (Optional but Recommended)

In Cloudflare Dashboard:

SSL/TLS → Edge Certificates → Enable HTTP/3

Cloudflare handles QUIC at the edge automatically.
Your origin will typically receive HTTP/2 or HTTP/1.1.


# 🔒 Firewall Recommendation (Optional but Strongly Recommended)

Restrict inbound traffic on port 443
Allow only Cloudflare IP ranges.

This prevents direct access to your origin server.


# ✅ Final Production Checklist

[ ] Origin certificate installed
[ ] Private key secured
[ ] Server running on port 443
[ ] Cloudflare SSL mode = Full (strict)
[ ] DNS proxy enabled (Orange Cloud)
[ ] Firewall configured
[ ] HTTP/3 enabled (optional)

antinna.in is now securely configured behind Cloudflare.

