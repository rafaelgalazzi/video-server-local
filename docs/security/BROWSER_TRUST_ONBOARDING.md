# Browser Trust Onboarding

LocalStream uses a private node root certificate so browser access can remain encrypted and authenticated without Internet infrastructure. Export is available only on the trusted desktop and only after acknowledging that the complete SHA-256 fingerprint will be compared.

## Required Verification

1. On the LocalStream desktop, record the complete root fingerprint and export the public `.cer` file.
2. Transfer that public certificate using a trusted method.
3. Before installation, inspect the certificate and compare its complete SHA-256 fingerprint with the trusted-desktop value. Stop if any character differs.
4. Install it only into the intended device/user trusted-root store, restart the browser, and open the configured HTTPS endpoint.
5. Remove the certificate if the node identity is reset, retired, or suspected compromised.

Never bypass a certificate warning. Never install a certificate obtained from the unauthenticated LAN endpoint. LocalStream does not remotely distribute or automatically install its root.

## Platform Notes

- Windows current-user trust: import the certificate into `Trusted Root Certification Authorities` for the current user. The workflow text and exported DER were reviewed on Windows; an interactive trust-store installation was **not verified** because LocalStream intentionally does not perform it.
- macOS: add the certificate to the login keychain and explicitly set trust. **Not verified on macOS.**
- Linux: distribution and browser stores differ; follow the administrator documentation for the specific trust store. **Not verified on Linux.**
- Firefox may use its own authority store depending on platform and policy. **Not verified.**

Platform UI labels change over time. Fingerprint comparison and refusal to bypass warnings are invariant.
