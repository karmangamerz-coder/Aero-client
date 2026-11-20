# Contributing

Contributions are welcome through public issues and pull requests. Follow the
[Code of Conduct](CODE_OF_CONDUCT.md). Discuss substantial changes in an issue first;
small fixes need no prior approval. Maintainers review technical merit openly.

1. Fork the public repository and create a focused branch.
2. Use JDK 21 and the checked-in wrapper; run `./gradlew clean build`.
3. Add meaningful tests for behavior and regression fixes, plus documentation.
4. Add an ADR for major architectural choices and update CHANGELOG's Unreleased section.
5. Open a PR describing the problem, behavior, tests and limitations.

Your original contributions are licensed GPL-3.0-only. Add an SPDX header to new
source files. Submit only code you are entitled to contribute; describe provenance
of adapted open-source code and preserve attribution. Never use leaked or proprietary
Lunar Client or OneClient code. No contributor license assignment is required.

Dependencies require a recorded version, upstream URL, SPDX license, compatibility
review, notices, vulnerability assessment and redistribution obligations in
THIRD_PARTY_LICENSES.md. Do not assume a license label alone establishes compatibility.
Pin versions; reject dynamic/SNAPSHOT dependencies. Generate locks and verification
metadata with `./gradlew build --write-locks --write-verification-metadata sha256`,
then independently verify new hashes against trusted upstream artifacts before review.
Do not blindly trust newly generated checksums.

Never commit credentials, access/refresh tokens, signing material, private logs or
personal information. Use environment variables for build credentials and OS secure
storage for future runtime tokens. Review `git diff --cached` before committing.
Security issues follow SECURITY.md instead of public issue reports.
