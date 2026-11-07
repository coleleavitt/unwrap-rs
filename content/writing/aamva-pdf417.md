+++
title: CVE-2025-31337 and CVE-2025-31336: ID scanners that accept a birth year of 1500
date: 2026-11-07
description: Driver's license barcodes are unsigned. Two scanner vendors validated their format but not whether the data could be true.
+++

Every US driver's license carries a PDF417 barcode with the holder's data, laid out by the AAMVA DL/ID standard. Bars, clubs and businesses scan it to check age and to "detect fake IDs". The data in that barcode is not signed. Anyone can print a barcode with whatever name, birth date and license number they like, and a scanner can only judge it by what the data says.

So the question I tested in 2025 was simple: do these products check whether the data could be true, or only whether it is formatted correctly?

## What I found

I generated synthetic barcodes with a small Rust tool and scanned them with several verification products. The results were consistent:

- **Impossible birth dates passed.** IDScan.net's VeriScan accepted a license whose holder was born in the year 1500. TokenWorks devices accepted birth years from the 1400s onward.
- **Only the format was checked.** A barcode that follows the AAMVA layout, with a real state issuer number (for example California's IIN 636014) and synthetic data, was treated as valid. Nothing checked the data against the issuing state.
- **Repeated identities went unnoticed.** The same license number scanned with different names was accepted each time.
- **State-specific markers were not cross-checked.** TokenWorks accepted forged ZC, ZF and ZG subfile markers with calculated offsets.

The same pattern showed up across licenses from Arizona, California, Florida, Georgia, New Jersey, South Carolina and Texas, and in Missouri's own ShowMeID verification app.

Two CVEs were assigned:

- [CVE-2025-31337](https://www.cve.org/CVERecord?id=CVE-2025-31337): IDScan.net (VeriScan, VSCloud and apps using its API)
- [CVE-2025-31336](https://www.cve.org/CVERecord?id=CVE-2025-31336): TokenWorks (IDentiFake Plus, AgeVisor and IDVisor Sentry)

## Disclosure

| Date | Event |
|---|---|
| 2025-03-22 | CVE request to MITRE |
| 2025-03-25 | Proof of concept sent to IDScan.net; acknowledged |
| 2025-03-28 | TokenWorks acknowledges the report |
| 2025-04-03 | CISA opens VINCE case VU#396042 |
| 2025-04-09 to 16 | Technical discussion with the vendors |
| 2025-06-18 | CISA closes the case |
| 2025-07-01 | Public disclosure |

## The vendors' answer

IDScan.net's first reply: "After investigating this, it looks like you are just spoofing the barcode data. This is a common standard." Later replies argued that a parser only reads data: "Where do you draw the line, especially in just a parser". On repeated license numbers: "We would not know if the first or second scan was real". TokenWorks described the issue as a DMV implementation problem.

CISA closed the case on June 18, 2025: "CISA has decided that this is not a vulnerability as it is an understood risk in the AAMVA DL Standard." MITRE published both CVEs anyway.

## Where I disagree

Part of this really is the standard. AAMVA barcodes have no signature, so a scanner cannot prove a barcode came from a DMV. That much is an understood risk.

But the products are not sold as parsers. They are sold to detect fake IDs, with "security checks". Some checks need no signature at all:

- a birth date more than about 120 years ago is not a real person;
- one license number appearing with several names is a pattern worth flagging;
- an issue date in the future is wrong.

Format compliance is not validation. If a product's job is to tell a real license from a fake one, accepting a customer born in 1500 is a bug in that product, whatever the standard says.

## What helps

- **If you rely on a scanner**, treat a clean barcode scan as "well formed", not "authentic". Check the physical card too.
- **If you build one**, add plausibility checks on dates and repeated numbers. They cost almost nothing.
- **The real fix** is signed credentials. Mobile driver's licenses (ISO/IEC 18013-5) carry issuer signatures that a verifier can check, which a printed barcode never could.

## Evidence

The proof-of-concept code, the scan screenshots and the disclosure record are in [AAMVA-PDF417-Vulnerability-Research](https://github.com/coleleavitt/AAMVA-PDF417-Vulnerability-Research). Eight of the screenshots there, including the Missouri ShowMeID result, have detached signatures from my key `2EFA A479 1439 CF35 47A9 6680 1ECC 2986 AF90 3402`, which [this site publishes](/pgp.asc). All test data was synthetic.
